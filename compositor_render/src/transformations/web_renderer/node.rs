use std::{collections::HashMap, sync::Arc};

use compositor_common::scene::{NodeId, Resolution};
use log::error;

use crate::renderer::{
    texture::{utils::pad_to_256, NodeTexture},
    RenderCtx,
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
            let size = texture.size();
            let size = (4 * pad_to_256(size.width) * size.height) as u64;

            let recreate_buffer = match self.buffers.get(id) {
                Some(buffer) => buffer.size() != size,
                None => true,
            };

            if recreate_buffer {
                self.buffers.insert(
                    (*id).clone(),
                    Arc::new(texture.new_download_buffer(ctx.wgpu_ctx)),
                );
            }
        }

        if let Err(err) = self.renderer.render(ctx, sources, &self.buffers, target) {
            error!("Failed to run web render: {err}");
        }
    }

    pub fn resolution(&self) -> Resolution {
        self.renderer.resolution()
    }
}
