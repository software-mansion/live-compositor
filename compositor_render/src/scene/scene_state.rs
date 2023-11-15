use compositor_common::scene::{OutputId, Resolution};

use super::{
    layout::SizedLayoutComponent, BuildSceneError, Component, InputStreamComponent,
    LayoutComponent, LayoutNode, Node, NodeKind, ShaderComponent,
};

pub(crate) struct SceneState {
    outputs: Vec<OutputScene>,
}

pub struct OutputScene {
    pub output_id: OutputId,
    pub root: Component,
    pub resolution: Resolution,
}

pub(crate) struct OutputNode {
    pub(crate) output_id: OutputId,
    pub(crate) node: Node,
    pub(crate) resolution: Resolution,
}

impl SceneState {
    pub fn new() -> Self {
        Self { outputs: vec![] }
    }

    pub(crate) fn update_scene(
        &mut self,
        outputs: Vec<OutputScene>,
    ) -> Result<Vec<OutputNode>, BuildSceneError> {
        let nodes = outputs
            .iter()
            .map(|output| {
                Ok(OutputNode {
                    output_id: output.output_id.clone(),
                    node: output
                        .root
                        .base_node()?
                        .build_node(Some(output.resolution))?,
                    resolution: output.resolution,
                })
            })
            .collect::<Result<_, _>>()?;
        self.outputs = outputs;
        Ok(nodes)
    }
}

/// Intermediate representation of a node tree while it's being constructed.
pub(super) enum BaseNode {
    InputStream(InputStreamComponent),
    Shader {
        shader: ShaderComponent,
        children: Vec<BaseNode>,
    },
    Layout {
        root: LayoutComponent,
        children: Vec<BaseNode>,
    },
}

impl BaseNode {
    fn build_node(self, resolution: Option<Resolution>) -> Result<Node, BuildSceneError> {
        let resolution = match resolution {
            Some(resolution) => resolution,
            None => self.node_size()?,
        };
        match self {
            BaseNode::InputStream(input) => Ok(Node {
                kind: NodeKind::InputStream(input),
                children: vec![],
            }),
            BaseNode::Shader { shader, children } => Ok(Node {
                kind: NodeKind::Shader(shader),
                children: children
                    .into_iter()
                    .map(|node| node.build_node(None))
                    .collect::<Result<_, _>>()?,
            }),
            BaseNode::Layout { root, children } => Ok(Node {
                kind: NodeKind::Layout(LayoutNode {
                    root: SizedLayoutComponent::new(root, resolution),
                }),
                children: children
                    .into_iter()
                    .map(|node| node.build_node(None))
                    .collect::<Result<_, _>>()?,
            }),
        }
    }

    fn node_size(&self) -> Result<Resolution, BuildSceneError> {
        match self {
            BaseNode::InputStream(input) => Ok(input.size.unwrap_or(Resolution {
                width: 1,
                height: 1,
            })),
            BaseNode::Shader {
                shader,
                children: _,
            } => Ok(shader.size),
            BaseNode::Layout { root, children: _ } => {
                let width = root.width();
                let height = root.height();
                if let (Some(width), Some(height)) = (width, height) {
                    Ok(Resolution { width, height })
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
