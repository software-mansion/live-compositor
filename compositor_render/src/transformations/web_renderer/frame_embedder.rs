use crate::gpu_shader::{CreateShaderError, GpuShader};
use crate::renderer::texture::{NodeTexture, NodeTextureState};
use crate::renderer::{RegisterCtx, WgpuCtx};
use crate::transformations::web_renderer::browser_client::BrowserClient;
use crate::transformations::web_renderer::chromium_sender::ChromiumSender;
use crate::transformations::web_renderer::FrameBytes;
use bytes::Bytes;
use compositor_chromium::cef;
use compositor_common::scene::{NodeId, Resolution};
use crossbeam_channel::bounded;
use log::error;
use shared_memory::ShmemError;
use std::sync::{Arc, Mutex};

pub(super) type FrameTransforms = Arc<Mutex<Bytes>>;

pub(super) struct FrameEmbedder {
    wgpu_ctx: Arc<WgpuCtx>,
    chromium_sender: ChromiumSender,
    frame_transforms: FrameTransforms,
    shader: GpuShader,
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl FrameEmbedder {
    pub fn new(
        ctx: &RegisterCtx,
        frame_bytes: FrameBytes,
        url: String,
        resolution: Resolution,
    ) -> Result<Self, FrameEmbedderError> {
        let frame_transforms = Arc::new(Mutex::new(Bytes::new()));
        let client = BrowserClient::new(frame_bytes, frame_transforms.clone(), resolution);
        let chromium_sender = ChromiumSender::new(ctx, url, client);

        let shader = GpuShader::new(
            &ctx.wgpu_ctx,
            include_str!("../builtin/apply_transformation_matrix.wgsl").into(),
        )?;
        let (buffer, bind_group) = Self::create_buffer(&ctx.wgpu_ctx, 1);

        Ok(Self {
            wgpu_ctx: ctx.wgpu_ctx.clone(),
            chromium_sender,
            frame_transforms,
            shader,
            buffer,
            bind_group,
        })
    }

    /// Embed sources by rendering them directly onto the target texture
    pub fn embed(&mut self, sources: &[(&NodeId, &NodeTexture)], target: &NodeTextureState) {
        self.chromium_sender.request_frame_positions(sources);

        let frame_positions = self.frame_transforms.lock().unwrap().clone();
        if frame_positions.is_empty() {
            return;
        }

        self.update_buffer(&frame_positions);
        self.shader
            .render(&self.bind_group, sources, target, Default::default(), None);
    }

    /// Send sources to chromium and render them on canvases via JS API
    pub fn native_embed(
        &mut self,
        node_id: NodeId,
        sources: &[(&NodeId, &NodeTexture)],
        buffers: &[Arc<wgpu::Buffer>],
    ) -> Result<(), FrameEmbedderError> {
        self.chromium_sender
            .alloc_shared_memory(node_id.clone(), sources);
        self.copy_sources_to_buffers(sources, buffers)?;

        let mut pending_downloads = Vec::new();
        for (source_idx, ((_, texture), buffer)) in sources.iter().zip(buffers).enumerate() {
            let Some(texture_state) = texture.state() else {
                continue;
            };
            let size = texture_state.rgba_texture().size();
            pending_downloads.push(self.copy_buffer_to_shmem(
                node_id.clone(),
                source_idx,
                size,
                buffer.clone(),
            ));
        }

        self.wgpu_ctx.device.poll(wgpu::Maintain::Wait);

        for pending in pending_downloads {
            pending()?;
        }

        self.chromium_sender.embed_sources(node_id, sources);
        Ok(())
    }

    fn update_buffer(&mut self, data: &[u8]) {
        if self.buffer.size() as usize != data.len() {
            let (buffer, bind_group) = Self::create_buffer(&self.wgpu_ctx, data.len());
            self.buffer = buffer;
            self.bind_group = bind_group;
        }

        self.wgpu_ctx.queue.write_buffer(&self.buffer, 0, data);
    }

    fn create_buffer(ctx: &WgpuCtx, size: usize) -> (wgpu::Buffer, wgpu::BindGroup) {
        let buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Frame transforms"),
            size: size as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });
        let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Frame embedder bind group"),
            layout: &ctx.shader_parameters_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        (buffer, bind_group)
    }

    pub fn request_frame_positions(&self, sources: &[(&NodeId, &NodeTexture)]) {
        self.chromium_sender.request_frame_positions(sources);
    }

    fn copy_sources_to_buffers(
        &self,
        sources: &[(&NodeId, &NodeTexture)],
        buffers: &[Arc<wgpu::Buffer>],
    ) -> Result<(), FrameEmbedderError> {
        let mut encoder = self
            .wgpu_ctx
            .device
            .create_command_encoder(&Default::default());

        for ((_, texture), buffer) in sources.iter().zip(buffers) {
            let Some(texture_state) = texture.state() else {
                continue;
            };
            texture_state
                .rgba_texture()
                .copy_to_buffer(&mut encoder, buffer);
        }
        self.wgpu_ctx.queue.submit(Some(encoder.finish()));

        Ok(())
    }

    fn copy_buffer_to_shmem(
        &self,
        node_id: NodeId,
        source_idx: usize,
        size: wgpu::Extent3d,
        source: Arc<wgpu::Buffer>,
    ) -> impl FnOnce() -> Result<(), FrameEmbedderError> + '_ {
        let (s, r) = bounded(1);
        source
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                if let Err(err) = s.send(result) {
                    error!("channel send error: {err}")
                }
            });

        move || {
            r.recv().unwrap()?;

            self.chromium_sender
                .update_shared_memory(node_id, source_idx, source.clone(), size);
            source.unmap();

            Ok(())
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FrameEmbedderError {
    #[error(transparent)]
    CreateShaderFailed(#[from] CreateShaderError),

    #[error("Failed to create shared memory")]
    CreateSharedMemory(#[from] ShmemError),

    #[error("Failed to download source frame")]
    DownloadFrame(#[from] wgpu::BufferAsyncError),

    #[error("Browser is no longer alive")]
    BrowserNotAlive(#[from] cef::BrowserError),

    #[error("Could not send IPC message")]
    MessageNotSent(#[from] cef::FrameError),

    #[error("Download buffer does not exist")]
    ExpectDownloadBuffer,
}
