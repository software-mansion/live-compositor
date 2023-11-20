use std::{sync::Arc, time::Duration};

use crate::{
    renderer::{render_graph::NodeId, RenderCtx},
    wgpu::texture::NodeTexture,
};

mod layout_renderer;
mod params;
mod shader;
mod transformation_matrices;

use compositor_common::{scene::Resolution, util::colors::RGBAColor};

use self::{
    params::{LayoutNodeParams, ParamsBuffer},
    shader::LayoutShader,
};

pub(crate) use layout_renderer::LayoutRenderer;

pub(crate) trait LayoutProvider: Send {
    fn layouts(&mut self, pts: Duration, inputs: &[Option<Resolution>]) -> Vec<Layout>;
    fn resolution(&self, pts: Duration) -> Resolution;
}

pub(crate) struct LayoutNode {
    layout_provider: Box<dyn LayoutProvider>,
    shader: Arc<LayoutShader>,
    params: ParamsBuffer,
}

#[derive(Debug, Clone)]
pub struct Crop {
    pub top: f32,
    pub left: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone)]
pub struct Layout {
    pub top: f32,
    pub left: f32,
    pub width: f32,
    pub height: f32,
    pub rotation_degrees: f32,
    pub content: LayoutContent,
}

#[derive(Debug, Clone)]
pub struct NestedLayout {
    pub(crate) layout: Layout,
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

impl NestedLayout {
    pub fn flatten(mut self, child_index_offset: usize) -> Vec<Layout> {
        let mut child_index_offset = child_index_offset;
        if let LayoutContent::ChildNode { index, .. } = self.layout.content {
            // TODO fix this
            self.layout.content = LayoutContent::ChildNode {
                index: index + child_index_offset,
                crop: Crop {
                    top: 0.0,
                    left: 0.0,
                    width: 1920.0,
                    height: 1080.0,
                },
            };
            child_index_offset += 1
        }
        let children: Vec<_> = self
            .children
            .into_iter()
            .flat_map(|child| {
                let child_nodes_count = child.child_nodes_count;
                let layouts = child.flatten(child_index_offset);
                child_index_offset += child_nodes_count;
                layouts
            })
            .map(|layout| Layout {
                top: layout.top + self.layout.top,
                left: layout.left + self.layout.left,
                width: layout.width,
                height: layout.height,
                rotation_degrees: layout.rotation_degrees + self.layout.rotation_degrees, // TODO: not exactly correct
                content: layout.content,
            })
            .filter(|layout| {
                !matches!(
                    layout.content,
                    LayoutContent::None | LayoutContent::Color(RGBAColor(0, 0, 0, 0))
                )
            })
            .collect();
        [vec![self.layout], children].concat()
    }
}

#[derive(Debug, Clone)]
pub enum LayoutContent {
    Color(RGBAColor),
    ChildNode {
        /// input pad index
        index: usize,
        crop: Crop,
    },
    None,
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
        let layouts = self.layout_provider.layouts(pts, &input_resolutions);
        let layout_count = layouts.len();
        let output_resolution = self.layout_provider.resolution(pts);

        let params: Vec<LayoutNodeParams> = layouts
            .iter()
            .map(|layout| {
                let (texture_id, background_color, input_resolution) = match layout.content {
                    LayoutContent::Color(color) => (
                        -1,
                        color,
                        Resolution {
                            width: 0,
                            height: 0,
                        },
                    ),
                    LayoutContent::ChildNode { index, .. } => {
                        let layout_resolution = Resolution {
                            width: layout.width as usize,
                            height: layout.height as usize,
                        };

                        (
                            index as i32,
                            RGBAColor(0, 0, 0, 0),
                            input_resolutions
                                .get(index)
                                .unwrap_or(&Some(layout_resolution))
                                .unwrap_or(layout_resolution),
                        )
                    }
                    LayoutContent::None => (
                        -1,
                        RGBAColor(0, 0, 0, 0),
                        Resolution {
                            width: 0,
                            height: 0,
                        },
                    ),
                };
                
                // TODO remove this - it's hard coded for crop example
                let cropped_layout = match layout.content {
                    LayoutContent::Color(_) | LayoutContent::None => layout.clone(),
                    LayoutContent::ChildNode { index, .. } => Layout {
                        content: LayoutContent::ChildNode {
                            index,
                            crop: Crop {
                                left: 960.0,
                                top: 540.0,
                                width: 960.0,
                                height: 540.0,
                            },
                        },
                        ..layout.clone()
                    },
                };

                LayoutNodeParams {
                    texture_id,
                    background_color,
                    transform_vertices_matrix: cropped_layout
                        .transform_vertices_matrix(&output_resolution),
                    transform_texture_coords_matrix: cropped_layout
                        .transform_texture_coords_matrix(&input_resolution),
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
