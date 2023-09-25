use crate::renderer::texture::NodeTexture;
use crate::renderer::{RegisterCtx, WgpuCtx};
use crate::transformations::web_renderer::chromium_sender::ChromiumSender;
use bytes::{Bytes, BytesMut};
use compositor_common::renderer_spec::WebEmbeddingMethod;
use compositor_common::scene::NodeId;
use crossbeam_channel::bounded;
use log::error;
use nalgebra_glm::Mat4;
use std::sync::Arc;

pub(super) struct EmbeddingHelper {
    wgpu_ctx: Arc<WgpuCtx>,
    chromium_sender: ChromiumSender,
    embedding_method: WebEmbeddingMethod,
}

impl EmbeddingHelper {
    pub fn new(
        ctx: &RegisterCtx,
        chromium_sender: ChromiumSender,
        embedding_method: WebEmbeddingMethod,
    ) -> Self {
        Self {
            wgpu_ctx: ctx.wgpu_ctx.clone(),
            chromium_sender,
            embedding_method,
        }
    }

    pub fn prepare_embedding(
        &self,
        node_id: &NodeId,
        sources: &[(&NodeId, &NodeTexture)],
        buffers: &[Arc<wgpu::Buffer>],
    ) -> Result<(), EmbedError> {
        match self.embedding_method {
            WebEmbeddingMethod::ChromiumEmbedding => {
                self.chromium_embedding(node_id, sources, buffers)?
            }
            WebEmbeddingMethod::EmbedOnTop | WebEmbeddingMethod::EmbedBelow => {
                self.chromium_sender.request_frame_positions(sources)
            }
        }

        Ok(())
    }

    /// Send sources to chromium and render them on canvases via JS API
    fn chromium_embedding(
        &self,
        node_id: &NodeId,
        sources: &[(&NodeId, &NodeTexture)],
        buffers: &[Arc<wgpu::Buffer>],
    ) -> Result<(), EmbedError> {
        self.chromium_sender
            .ensure_shared_memory(node_id.clone(), sources);
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

        self.chromium_sender.embed_sources(node_id.clone(), sources);
        Ok(())
    }

    fn copy_sources_to_buffers(
        &self,
        sources: &[(&NodeId, &NodeTexture)],
        buffers: &[Arc<wgpu::Buffer>],
    ) -> Result<(), EmbedError> {
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
    ) -> impl FnOnce() -> Result<(), EmbedError> + '_ {
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

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub(super) struct TextureInfo {
    is_website_texture: i32,
    _padding: [f32; 3],
    transformation_matrix: [[f32; 4]; 4],
}

impl TextureInfo {
    pub fn website() -> Bytes {
        let texture_info = Self {
            is_website_texture: 1,
            _padding: Default::default(),
            transformation_matrix: Mat4::identity().transpose().into(),
        };

        Bytes::copy_from_slice(bytemuck::cast_slice(&[texture_info]))
    }

    pub fn sources(transformation_matrices: &[Mat4]) -> Bytes {
        let mut textures_info = BytesMut::new();
        for transform in transformation_matrices {
            let info = Self {
                is_website_texture: 0,
                _padding: Default::default(),
                transformation_matrix: transform.transpose().into(),
            };
            textures_info.extend(bytemuck::cast_slice(&[info]));
        }

        textures_info.freeze()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EmbedError {
    #[error("Failed to download source frame")]
    DownloadFrame(#[from] wgpu::BufferAsyncError),
}
