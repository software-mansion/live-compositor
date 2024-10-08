use std::{sync::Arc, time::Duration};

use crate::{
    scene::{RGBAColor, Size},
    state::RenderCtx,
    wgpu::texture::NodeTexture,
    Resolution,
};

mod flatten;
mod layout_renderer;
mod params;
mod shader;
mod transformation_matrices;

use self::shader::LayoutShader;

pub(crate) use layout_renderer::LayoutRenderer;
pub(crate) use transformation_matrices::{vertices_transformation_matrix, Position};

use log::error;

pub(crate) trait LayoutProvider: Send {
    fn layouts(&mut self, pts: Duration, inputs: &[Option<Resolution>]) -> NestedLayout;
    fn resolution(&self, pts: Duration) -> Resolution;
}

pub(crate) struct LayoutNode {
    layout_provider: Box<dyn LayoutProvider>,
    shader: Arc<LayoutShader>,
}

#[derive(Debug, Clone)]
pub struct Crop {
    pub top: f32,
    pub left: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone)]
pub struct BorderRadius {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}

#[derive(Debug, Clone)]
pub struct ParentBodererRadius {
    pub radius: BorderRadius,
    // position of parent on the output frame
    pub top: f32,
    pub left: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone)]
struct RenderLayout {
    top: f32,
    left: f32,
    width: f32,
    height: f32,
    rotation_degrees: f32,
    border_radius: BorderRadius,
    parent_border_radiuses: Vec<ParentBodererRadius>,
    content: RenderLayoutContent,
}

#[derive(Debug, Clone)]
enum RenderLayoutContent {
    Color {
        color: RGBAColor,
        border_color: RGBAColor,
        border_width: f32,
    },
    ChildNode {
        index: usize,
        crop: Crop,
        border_color: RGBAColor,
        border_width: f32,
    },
    #[allow(dead_code)]
    BoxShadow {
        color: RGBAColor,
        blur_radius: f32,
    },
}

#[derive(Debug, Clone)]
pub enum LayoutContent {
    Color(RGBAColor),
    ChildNode { index: usize, size: Size },
    None,
}

#[derive(Debug, Clone)]
pub struct NestedLayout {
    pub top: f32,
    pub left: f32,
    pub width: f32,
    pub height: f32,
    pub rotation_degrees: f32,
    /// scale will affect content/children, but not the properties of current layout like
    /// top/left/widht/height
    pub scale_x: f32,
    pub scale_y: f32,
    /// Crop is applied before scaling.
    pub crop: Option<Crop>,
    pub content: LayoutContent,

    pub(crate) children: Vec<NestedLayout>,
    /// Describes how many children of this component are nodes. This value also
    /// counts `layout` if its content is a `LayoutContent::ChildNode`.
    ///
    /// `child_nodes_count` is not necessarily equal to number of `LayoutContent::ChildNode` in
    /// a sub-tree. For example, if we have a component that conditionally shows one
    /// of its children then child_nodes_count will count all of those components even
    /// though only one of those children will be present in the layouts tree.
    pub(crate) child_nodes_count: usize,
}

impl LayoutNode {
    pub fn new(ctx: &RenderCtx, layout_provider: Box<dyn LayoutProvider>) -> Self {
        let shader = ctx.renderers.layout.0.clone();

        Self {
            layout_provider,
            shader,
        }
    }

    pub fn render(
        &mut self,
        ctx: &RenderCtx,
        sources: &[&NodeTexture],
        target: &mut NodeTexture,
        pts: Duration,
    ) {
        let input_resolutions: Vec<Option<Resolution>> = sources
            .iter()
            .map(|node_texture| node_texture.resolution())
            .collect();
        let output_resolution = self.layout_provider.resolution(pts);
        let layouts = self
            .layout_provider
            .layouts(pts, &input_resolutions)
            .flatten(&input_resolutions, output_resolution);

        let textures: Vec<Option<&NodeTexture>> = layouts
            .iter()
            .map(|layout| match layout.content {
                RenderLayoutContent::BoxShadow { .. } => None,
                RenderLayoutContent::Color { .. } => None,
                RenderLayoutContent::ChildNode { index, .. } => match sources.get(index) {
                    Some(node_texture) => Some(*node_texture),
                    None => {
                        error!("Invalid source index in layout");
                        None
                    }
                },
            })
            .collect();

        let target = target.ensure_size(ctx.wgpu_ctx, output_resolution);
        self.shader
            .render(ctx.wgpu_ctx, output_resolution, layouts, &textures, target);
    }
}

impl NestedLayout {
    /// NestedLayout that won't ever be rendered. It's intended to be optimized out
    /// in the flattening process. Its only purpose is to keep track of child nodes that are not
    /// currently used so the index offset can be calculated correctly.
    pub(crate) fn child_nodes_placeholder(child_nodes_count: usize) -> Self {
        Self {
            top: 0.0,
            left: 0.0,
            width: 0.0,
            height: 0.0,
            rotation_degrees: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            crop: None,
            content: LayoutContent::None,
            children: vec![],
            child_nodes_count,
        }
    }
}
