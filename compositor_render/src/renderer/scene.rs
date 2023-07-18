use std::{collections::HashMap, sync::Arc};

use compositor_common::scene::{
    InputId, InputSpec, NodeId, Resolution, SceneSpec, ShaderParams, TransformNodeSpec,
    TransformParams,
};
use log::error;

use crate::{
    registry::GetError,
    transformations::{shader::Shader, web_renderer::WebRenderer},
};

use super::{
    texture::{OutputTexture, Texture},
    RenderCtx,
};

pub struct InputNode {}

pub enum TransformNode {
    Shader {
        params: HashMap<String, ShaderParams>,
        shader: Arc<Shader>,
    },
    WebRenderer {
        renderer: Arc<WebRenderer>,
    },
}

impl TransformNode {
    fn new(ctx: &RenderCtx, spec: &TransformParams) -> Result<Self, GetError> {
        match spec {
            TransformParams::WebRenderer { renderer_id } => Ok(TransformNode::WebRenderer {
                renderer: ctx.web_renderers.get(renderer_id)?.clone(),
            }),
            TransformParams::Shader {
                shader_id,
                shader_params,
            } => Ok(TransformNode::Shader {
                params: shader_params.clone(),
                shader: ctx.shader_transforms.get(shader_id)?,
            }),
        }
    }
    pub fn render(&self, ctx: &RenderCtx, sources: &Vec<(NodeId, &Texture)>, target: &Texture) {
        match self {
            TransformNode::Shader {
                params: _,
                shader: _,
            } => {
                // shader.render(sources, target, other_args);
                error!("mocked shader render")
            }
            TransformNode::WebRenderer { renderer } => {
                if let Err(err) = renderer.render(ctx, sources, target) {
                    error!("Render operation failed {err}");
                }
            }
        }
    }
}

pub enum Node {
    Input {
        node_id: InputId,
        output: Texture,
        resolution: Resolution,
        yuv_textures: [Texture; 3],
    },
    Transformation {
        node_id: NodeId,
        output: Texture,
        resolution: Resolution,
        inputs: Vec<Arc<Node>>,
        node: TransformNode,
    },
}

impl Node {
    pub fn new_transform(
        ctx: &RenderCtx,
        spec: &TransformNodeSpec,
        inputs: Vec<Arc<Node>>,
    ) -> Result<Self, GetError> {
        let node = TransformNode::new(ctx, &spec.transform_params)?;
        let output = Texture::new_rgba(&ctx.wgpu_ctx, &spec.resolution);
        Ok(Self::Transformation {
            node_id: spec.node_id.clone(),
            node,
            resolution: spec.resolution.clone(),
            inputs,
            output,
        })
    }

    pub fn new_input(ctx: &RenderCtx, spec: &InputSpec) -> Result<Self, GetError> {
        let output = Texture::new_rgba(&ctx.wgpu_ctx, &spec.resolution);

        Ok(Self::Input {
            node_id: spec.input_id.clone(),
            output,
            resolution: spec.resolution.clone(),
            yuv_textures: Texture::new_yuv_textures(&ctx.wgpu_ctx, &spec.resolution),
        })
    }

    pub fn inputs(&self) -> Vec<Arc<Node>> {
        match &self {
            Node::Input { .. } => vec![],
            Node::Transformation { inputs, .. } => inputs.clone(),
        }
    }

    pub fn output(&self) -> &Texture {
        match &self {
            Node::Input { output, .. } => output,
            Node::Transformation { output, .. } => output,
        }
    }

    pub fn id(&self) -> NodeId {
        match &self {
            Node::Input { node_id, .. } => node_id.0.clone(),
            Node::Transformation { node_id, .. } => node_id.clone(),
        }
    }

    pub fn resolution(&self) -> Resolution {
        match &self {
            Node::Input { resolution, .. } => resolution.clone(),
            Node::Transformation { resolution, .. } => resolution.clone(),
        }
    }
}

pub struct Scene {
    pub outputs: HashMap<NodeId, (Arc<Node>, OutputTexture)>,
}

#[derive(Debug, thiserror::Error)]
pub enum SceneUpdateError {
    #[error("Failed to construct transform node")]
    TransformNodeError(GetError),

    #[error("Failed to construct input node")]
    InputNodeError(GetError),

    #[error("No spec for node with id {0}")]
    NoNodeWithIdError(Arc<str>),
}

impl Scene {
    pub fn empty() -> Self {
        Self {
            outputs: HashMap::new(),
        }
    }

    pub fn update(&mut self, ctx: &RenderCtx, spec: SceneSpec) -> Result<(), SceneUpdateError> {
        // TODO: If we want nodes to be stateful we could try reusing nodes instead
        // of recreating them on every scene update
        let mut new_nodes = HashMap::new();
        self.outputs = spec
            .outputs
            .iter()
            .map(|output| {
                let node = self.ensure_node(ctx, &output.input_pad, &spec, &mut new_nodes)?;
                let buffers = OutputTexture::new(&ctx.wgpu_ctx, &node.resolution());
                Ok((output.input_pad.clone(), (node, buffers)))
            })
            .collect::<Result<_, _>>()?;
        Ok(())
    }

    fn ensure_node(
        &mut self,
        ctx: &RenderCtx,
        node_id: &NodeId,
        spec: &SceneSpec,
        new_nodes: &mut HashMap<NodeId, Arc<Node>>,
    ) -> Result<Arc<Node>, SceneUpdateError> {
        // check if node already exists
        if let Some(already_existing_node) = new_nodes.get(&node_id) {
            return Ok(already_existing_node.clone());
        }

        // handle a case where node_id refers to transform node
        {
            let transform_spec = spec.transforms.iter().find(|node| &node.node_id == node_id);
            if let Some(transform) = transform_spec {
                let inputs = transform
                    .input_pads
                    .iter()
                    .map(|node_id| self.ensure_node(ctx, node_id, &spec, new_nodes))
                    .collect::<Result<_, _>>()?;
                let node = Node::new_transform(ctx, transform, inputs)
                    .map_err(SceneUpdateError::TransformNodeError)?;
                let node = Arc::new(node);
                new_nodes.insert(node_id.clone(), node.clone());
                return Ok(node);
            }
        }

        // handle a case where node_id refers to input node
        {
            let input_spec = spec.inputs.iter().find(|node| &node.input_id.0 == node_id);
            if let Some(input) = input_spec {
                let node = Node::new_input(ctx, input).map_err(SceneUpdateError::InputNodeError)?;
                let node = Arc::new(node);
                new_nodes.insert(node_id.clone(), node.clone());
                return Ok(node);
            }
        }

        Err(SceneUpdateError::NoNodeWithIdError(node_id.0.clone()))
    }
}
