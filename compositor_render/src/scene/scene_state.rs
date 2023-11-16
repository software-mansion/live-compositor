use std::{collections::HashMap, time::Duration};

use compositor_common::scene::{InputId, OutputId, Resolution};

use crate::renderer::renderers::Renderers;

use super::{
    image_component::StatefulImageComponent,
    input_stream_component::StatefulInputStreamComponent,
    layout::{LayoutNode, SizedLayoutComponent, StatefulLayoutComponent},
    shader_component::StatefulShaderComponent,
    BuildSceneError, ComponentId, Node, NodeParams, OutputScene, Position, Size, StatefulComponent,
};

pub(super) struct BuildStateTreeCtx<'a> {
    pub(super) prev_state: HashMap<ComponentId, &'a StatefulComponent>,
    pub(super) last_render_pts: Duration,
    pub(super) renderers: &'a Renderers,
    pub(super) input_resolutions: &'a HashMap<InputId, Resolution>,
}

pub(crate) struct SceneState {
    outputs: Vec<OutputSceneState>,
    last_pts: Duration,
    // Input resolutions from the last render
    input_resolutions: HashMap<InputId, Resolution>,
}

#[derive(Debug, Clone)]
struct OutputSceneState {
    output_id: OutputId,
    root: StatefulComponent,
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
            input_resolutions: HashMap::new(),
        }
    }

    pub(crate) fn register_render_event(
        &mut self,
        pts: Duration,
        input_resolutions: HashMap<InputId, Resolution>,
    ) {
        self.last_pts = pts;
        self.input_resolutions = input_resolutions;
        // TODO: pass input stream sizes and populate it in the ComponentState tree
    }

    pub(crate) fn update_scene(
        &mut self,
        outputs: Vec<OutputScene>,
        renderers: &Renderers,
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
            input_resolutions: &self.input_resolutions,
            renderers,
        };
        let output_states = outputs
            .into_iter()
            .map(|o| {
                Ok(OutputSceneState {
                    output_id: o.output_id,
                    root: o.root.stateful_component(&ctx)?,
                    resolution: o.resolution,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let nodes = output_states
            .iter()
            .map(|output| {
                Ok(OutputNode {
                    output_id: output.output_id.clone(),
                    node: output
                        .root
                        .intermediate_node()?
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
    InputStream(StatefulInputStreamComponent),
    Shader {
        shader: StatefulShaderComponent,
        children: Vec<IntermediateNode>,
    },
    Image(StatefulImageComponent),
    Layout {
        root: StatefulLayoutComponent,
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
                params: NodeParams::InputStream(input.component.input_id), // TODO: enforce resolution
                children: vec![],
            }),
            IntermediateNode::Shader { shader, children } => Ok(Node {
                params: NodeParams::Shader(shader.component, shader.shader), // TODO: enforce resolution
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
            IntermediateNode::Image(image) => Ok(Node {
                params: NodeParams::Image(image.image),
                children: vec![],
            }),
        }
    }

    fn node_size(&self, pts: Duration) -> Result<Size, BuildSceneError> {
        match self {
            IntermediateNode::InputStream(input) => Ok(input.size),
            IntermediateNode::Shader {
                shader,
                children: _,
            } => Ok(shader.component.size),
            IntermediateNode::Image(image) => Ok(image.size()),
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
    component: &'a StatefulComponent,
    components: &mut HashMap<ComponentId, &'a StatefulComponent>,
) {
    match component {
        StatefulComponent::InputStream(input) => {
            if let Some(id) = input.component_id() {
                components.insert(id.clone(), component);
            }
        }
        StatefulComponent::Shader(shader) => {
            if let Some(id) = shader.component_id() {
                components.insert(id.clone(), component);
            }
            for child in shader.children.iter() {
                gather_components_with_id(child, components);
            }
        }
        StatefulComponent::Image(image) => {
            if let Some(id) = image.component_id() {
                components.insert(id.clone(), component);
            }
        }
        StatefulComponent::Layout(layout) => {
            if let Some(id) = layout.component_id() {
                components.insert(id.clone(), component);
            }
            for child in layout.children() {
                gather_components_with_id(child, components);
            }
        }
    }
}
