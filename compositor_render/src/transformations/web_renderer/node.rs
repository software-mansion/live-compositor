use std::{collections::HashMap, sync::Arc};

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
    buffers: HashMap<NodeId, Arc<wgpu::Buffer>>,
}

impl WebRendererNode {
    pub fn new(renderer: Arc<WebRenderer>) -> Self {
        Self {
            renderer,
            buffers: HashMap::new(),
        }
    }

    pub fn render(
        &mut self,
        ctx: &mut RenderCtx,
        sources: &[(&NodeId, &NodeTexture)],
        target: &mut NodeTexture,
    ) {
        for (id, texture) in sources {
            let Some(texture_state) = texture.state() else {
                continue;
            };

            let texture = texture_state.rgba_texture();
            match self.buffers.get(id) {
                Some(buffer) => self.ensure_buffer_size(ctx.wgpu_ctx, id, buffer.size(), texture),
                None => self.create_insert_buffer(ctx.wgpu_ctx, (*id).clone(), texture),
            }
        }

        if let Err(err) = self.renderer.render(ctx, sources, &self.buffers, target) {
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

    fn ensure_buffer_size(
        &mut self,
        ctx: &WgpuCtx,
        node_id: &NodeId,
        buffer_size: u64,
        texture: &RGBATexture,
    ) {
        let texture_size = texture.size();
        let texture_size = (4 * pad_to_256(texture_size.width) * texture_size.height) as u64;
        if buffer_size != texture_size {
            self.create_insert_buffer(ctx, node_id.clone(), texture);
        }
    }

    fn create_insert_buffer(&mut self, ctx: &WgpuCtx, node_id: NodeId, texture: &RGBATexture) {
        self.buffers
            .insert(node_id, Arc::new(texture.new_download_buffer(ctx)));
    }
}
