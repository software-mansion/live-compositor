use std::{fs, io, sync::Arc};

use compositor_common::{
    error::ErrorStack,
    renderer_spec::FallbackStrategy,
    scene::{NodeId, Resolution},
};
use log::error;

use crate::renderer::{
    texture::{utils::pad_to_256, NodeTexture, RGBATexture},
    RenderCtx, WgpuCtx,
};

use super::WebRenderer;

pub struct WebRendererNode {
    renderer: Arc<WebRenderer>,
    node_id: NodeId,
    buffers: Vec<Arc<wgpu::Buffer>>,
}

impl WebRendererNode {
    pub fn new(
        renderer_id: &str,
        node_id: &NodeId,
        renderer: Arc<WebRenderer>,
    ) -> Result<Self, WebRendererNodeError> {
        Self::init_shared_memory_folder(renderer_id, node_id)?;

        Ok(Self {
            renderer,
            node_id: node_id.clone(),
            buffers: Vec::new(),
        })
    }

    pub fn render(
        &mut self,
        ctx: &mut RenderCtx,
        sources: &[(&NodeId, &NodeTexture)],
        target: &mut NodeTexture,
    ) {
        for (i, (_, texture)) in sources.iter().enumerate() {
            let Some(texture_state) = texture.state() else {
                continue;
            };

            let texture = texture_state.rgba_texture();
            match self.buffers.get_mut(i) {
                Some(buffer) => Self::ensure_buffer_size(ctx.wgpu_ctx, buffer, texture),
                None => self
                    .buffers
                    .push(Arc::new(texture.new_download_buffer(ctx.wgpu_ctx))),
            }
        }

        self.buffers.truncate(sources.len());

        if let Err(err) = self
            .renderer
            .render(ctx, &self.node_id, sources, &self.buffers, target)
        {
            error!(
                "Failed to run web render: {}",
                ErrorStack::new(&err).into_string()
            );
        }
    }

    pub fn resolution(&self) -> Resolution {
        self.renderer.resolution()
    }

    pub fn fallback_strategy(&self) -> FallbackStrategy {
        self.renderer.fallback_strategy()
    }

    fn ensure_buffer_size(ctx: &WgpuCtx, buffer: &mut Arc<wgpu::Buffer>, texture: &RGBATexture) {
        let texture_size = texture.size();
        let texture_size = (4 * pad_to_256(texture_size.width) * texture_size.height) as u64;
        if buffer.size() != texture_size {
            *buffer = Arc::new(texture.new_download_buffer(ctx));
        }
    }

    /// Creates folder for shared memory descriptors used by this node
    fn init_shared_memory_folder(
        renderer_id: &str,
        node_id: &NodeId,
    ) -> Result<(), WebRendererNodeError> {
        let path = WebRenderer::shared_memory_root_path(renderer_id).join(node_id.to_string());
        if path.exists() {
            return Ok(());
        }

        fs::create_dir_all(&path).map_err(WebRendererNodeError::CreateShmemFolderFailed)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WebRendererNodeError {
    #[error("Failed to create folder for shared memory")]
    CreateShmemFolderFailed(#[source] io::Error),
}
