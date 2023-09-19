use crate::gpu_shader::{CreateShaderError, GpuShader};
use crate::renderer::texture::{NodeTexture, NodeTextureState};
use crate::renderer::WgpuCtx;
use crate::transformations::web_renderer::browser::BrowserController;
use bytes::Bytes;
use compositor_chromium::cef;
use compositor_common::scene::NodeId;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub(super) type FrameTransforms = Arc<Mutex<Bytes>>;

pub(super) struct FrameEmbedder {
    ctx: Arc<WgpuCtx>,
    frame_transforms: FrameTransforms,
    shader: GpuShader,
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl FrameEmbedder {
    pub fn new(ctx: &Arc<WgpuCtx>) -> Result<Self, FrameEmbedderError> {
        let frame_transforms = Arc::new(Mutex::new(Bytes::new()));
        let shader = GpuShader::new(
            ctx,
            include_str!("../builtin/apply_transformation_matrix.wgsl").into(),
        )?;
        let (buffer, bind_group) = Self::create_buffer(ctx, 1);

        Ok(Self {
            ctx: ctx.clone(),
            frame_transforms,
            shader,
            buffer,
            bind_group,
        })
    }

    pub fn frame_transforms(&self) -> FrameTransforms {
        self.frame_transforms.clone()
    }

    pub fn embed(&mut self, sources: &[(&NodeId, &NodeTexture)], target: &NodeTextureState) {
        let frame_positions = self.frame_transforms.lock().unwrap().clone();
        if frame_positions.is_empty() {
            return;
        }

        self.update_buffer(&frame_positions);
        self.shader
            .render(&self.bind_group, sources, target, Default::default(), None);
    }

    fn update_buffer(&mut self, data: &[u8]) {
        if self.buffer.size() as usize != data.len() {
            let (buffer, bind_group) = Self::create_buffer(&self.ctx, data.len());
            self.buffer = buffer;
            self.bind_group = bind_group;
        }

        self.ctx
            .queue
            .write_buffer(&self.buffer, 0, data);
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
}

#[derive(Debug, thiserror::Error)]
pub enum FrameEmbedderError {
    #[error(transparent)]
    CreateShaderFailed(#[from] CreateShaderError),
}
