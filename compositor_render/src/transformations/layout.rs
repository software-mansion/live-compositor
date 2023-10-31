use std::{sync::Arc, time::Duration};

use crate::{renderer::RenderCtx, wgpu::texture::NodeTexture};

mod layout_renderer;
mod params;
mod shader;
mod transformation_matrix;

use compositor_common::{
    scene::{NodeId, Resolution},
    util::colors::RGBAColor,
};
pub(crate) use layout_renderer::LayoutRenderer;

use self::{
    params::{LayoutNodeParams, ParamsBuffer},
    shader::LayoutShader,
};

pub(crate) trait LayoutProvider: Send {
    fn layouts(&mut self, pts: Duration, inputs: Vec<Option<Resolution>>) -> Vec<Layout>;
    fn resolution(&self) -> Resolution;
}

pub(crate) struct LayoutNode {
    layout_provider: Box<dyn LayoutProvider>,
    shader: Arc<LayoutShader>,
    params: ParamsBuffer,
}

#[derive(Debug, Clone)]
pub struct Layout {
    pub top_left_corner: (f32, f32),
    pub width: f32,
    pub height: f32,
    pub rotation_degrees: f32,
    pub content: LayoutContent,
}

#[derive(Debug, Clone)]
pub enum LayoutContent {
    Color(RGBAColor),
    ChildNode(/* input pad index */ usize),
}

impl LayoutNode {
    pub fn new(ctx: &RenderCtx, layout_provider: Box<dyn LayoutProvider>) -> Self {
        let shader = ctx.renderers.layout.0.clone();

        Self {
            layout_provider,
            shader,
            params: ParamsBuffer::new(ctx.wgpu_ctx, vec![]),
        }
    }

    pub fn render(
        &mut self,
        ctx: &RenderCtx,
        sources: &[(&NodeId, &NodeTexture)],
        target: &mut NodeTexture,
        pts: Duration,
    ) {
        let input_resolutions: Vec<Option<Resolution>> = sources
            .iter()
            .map(|(_, node_texture)| node_texture.resolution())
            .collect();
        let layouts = self.layout_provider.layouts(pts, input_resolutions);
        let layout_count = layouts.len();
        let output_resolution = self.layout_provider.resolution();

        let params = layouts
            .iter()
            .map(|layout| {
                let (texture_id, background_color) = match layout.content {
                    LayoutContent::Color(color) => (-1, color),
                    LayoutContent::ChildNode(index) => (index as i32, RGBAColor(0, 0, 0, 0)),
                };
                LayoutNodeParams {
                    transformation_matrix: layout.transformation_matrix(output_resolution),
                    texture_id,
                    background_color,
                }
            })
            .collect();
        self.params.update(params, ctx.wgpu_ctx);

        let target = target.ensure_size(ctx.wgpu_ctx, output_resolution);
        self.shader.render(
            ctx.wgpu_ctx,
            self.params.bind_group(),
            sources,
            target,
            layout_count as u32,
        );
    }
}
