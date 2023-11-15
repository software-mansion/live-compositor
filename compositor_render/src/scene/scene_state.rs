use std::{collections::HashMap, time::Duration};

use compositor_common::scene::{OutputId, Resolution};

use super::{
    input_stream_component::InputStreamComponentState,
    layout::{LayoutComponentState, LayoutNode, SizedLayoutComponent},
    shader_component::ShaderComponentState,
    BuildSceneError, ComponentId, ComponentState, Node, NodeParams, OutputScene, Position, Size,
};

pub(super) struct BuildStateTreeCtx<'a> {
    pub(super) prev_state: HashMap<ComponentId, &'a ComponentState>,
    pub(super) last_render_pts: Duration,
}

pub(crate) struct SceneState {
    outputs: Vec<OutputSceneState>,
    last_pts: Duration,
}

#[derive(Debug, Clone)]
struct OutputSceneState {
    output_id: OutputId,
    root: ComponentState,
    resolution: Resolution,
}

pub(crate) struct OutputNode {
    pub(crate) output_id: OutputId,
    pub(crate) node: Node,
    pub(crate) resolution: Resolution,
}

impl SceneState {
    pub fn new() -> Self {
        Self {
            outputs: vec![],
            last_pts: Duration::ZERO,
        }
    }

    pub(crate) fn register_render_event(&mut self, pts: Duration) {
        self.last_pts = pts
        // TODO: pass input stream sizes and populate it in the ComponentState tree
    }

    pub(crate) fn update_scene(
        &mut self,
        outputs: Vec<OutputScene>,
    ) -> Result<Vec<OutputNode>, BuildSceneError> {
        let ctx = BuildStateTreeCtx {
            prev_state: self
                .outputs
                .iter()
                .flat_map(|o| {
                    let mut components = HashMap::new();
                    gather_components_with_id(&o.root, &mut components);
                    components
                })
                .collect(),
            last_render_pts: self.last_pts,
        };
        let output_states = outputs
            .into_iter()
            .map(|o| OutputSceneState {
                output_id: o.output_id,
                root: o.root.state_component(&ctx),
                resolution: o.resolution,
            })
            .collect::<Vec<_>>();
        let nodes = output_states
            .iter()
            .map(|output| {
                Ok(OutputNode {
                    output_id: output.output_id.clone(),
                    node: output
                        .root
                        .base_node()?
                        .build_tree(Some(output.resolution), self.last_pts)?,
                    resolution: output.resolution,
                })
            })
            .collect::<Result<_, _>>()?;
        self.outputs = output_states;
        Ok(nodes)
    }
}

/// Intermediate representation of a node tree while it's being constructed.
pub(super) enum IntermediateNode {
    InputStream(InputStreamComponentState),
    Shader {
        shader: ShaderComponentState,
        children: Vec<IntermediateNode>,
    },
    Layout {
        root: LayoutComponentState,
        children: Vec<IntermediateNode>,
    },
}

impl IntermediateNode {
    /// * `resolution` - Forces resolution of a node, primary use case for this
    ///   param is to force resolution on a top level component to match resolution
    ///   of an output stream. TODO: Currently only layouts respect that value
    /// * `pts` - PTS from the last render (this function is not called on render
    ///   so we can't have exact PTS here)
    fn build_tree(
        self,
        resolution: Option<Resolution>,
        pts: Duration,
    ) -> Result<Node, BuildSceneError> {
        let size = match resolution {
            Some(resolution) => resolution.into(),
            None => self.node_size(pts)?,
        };
        match self {
            IntermediateNode::InputStream(input) => Ok(Node {
                params: NodeParams::InputStream(input.component), // TODO: enforce resolution
                children: vec![],
            }),
            IntermediateNode::Shader { shader, children } => Ok(Node {
                params: NodeParams::Shader(shader.component), // TODO: enforce resolution
                children: children
                    .into_iter()
                    .map(|node| node.build_tree(None, pts))
                    .collect::<Result<_, _>>()?,
            }),
            IntermediateNode::Layout { root, children } => Ok(Node {
                params: NodeParams::Layout(LayoutNode {
                    root: SizedLayoutComponent::new(root, size),
                }),
                children: children
                    .into_iter()
                    .map(|node| node.build_tree(None, pts))
                    .collect::<Result<_, _>>()?,
            }),
        }
    }

    fn node_size(&self, pts: Duration) -> Result<Size, BuildSceneError> {
        match self {
            IntermediateNode::InputStream(input) => Ok(input.size.unwrap_or(Size {
                width: 1.0,
                height: 1.0,
            })),
            IntermediateNode::Shader {
                shader,
                children: _,
            } => Ok(shader.component.size),
            IntermediateNode::Layout { root, children: _ } => {
                let (width, height) = match root.position(pts) {
                    Position::Static { width, height } => (width, height),
                    // Technically absolute positioning is a bug here, but I think throwing error
                    // in this case would be to invasive. It's better to just ignore those values.
                    Position::Absolute(position) => (Some(position.width), Some(position.height)),
                };
                if let (Some(width), Some(height)) = (width, height) {
                    Ok(Size { width, height })
                } else {
                    Err(BuildSceneError::UnknownDimensionsForLayoutNodeRoot {
                        component: root.component_type(),
                        msg: match root.component_id() {
                            Some(id) => format!("Please provide width and height values for component with id \"{id}\""),
                            None => "Please provide width and height values.".to_string(),
                        },
                    })
                }
            }
        }
    }
}

fn gather_components_with_id<'a>(
    component: &'a ComponentState,
    components: &mut HashMap<ComponentId, &'a ComponentState>,
) {
    match component {
        ComponentState::InputStream(input) => {
            if let Some(id) = input.component_id() {
                components.insert(id.clone(), component);
            }
        }
        ComponentState::Shader(shader) => {
            if let Some(id) = shader.component_id() {
                components.insert(id.clone(), component);
            }
            for child in shader.children.iter() {
                gather_components_with_id(child, components);
            }
        }
        ComponentState::Layout(layout) => {
            if let Some(id) = layout.component_id() {
                components.insert(id.clone(), component);
            }
            for child in layout.children() {
                gather_components_with_id(child, components);
            }
        }
    }
}
