use std::{collections::HashMap, sync::Arc, time::Duration};

use compositor_common::{
    scene::{InputId, NodeId, NodeParams, NodeSpec, OutputId, Resolution, SceneSpec},
    SpecValidationError,
};
use log::error;

use crate::{
    registry::GetError,
    transformations::{
        builtin::transformations::BuiltinTransformations, image_renderer::ImageNode,
        shader::node::ShaderNode, text_renderer::TextRendererNode, web_renderer::WebRenderer,
    },
};

use super::{
    texture::{InputTexture, NodeTexture, OutputTexture},
    RenderCtx, WgpuError, WgpuErrorScope,
};

pub enum RenderNode {
    Shader(ShaderNode),
    Web { renderer: Arc<WebRenderer> },
    Text(TextRendererNode),
    Image(ImageNode),
    Builtin(ShaderNode),
    InputStream,
}

impl RenderNode {
    fn new(ctx: &RenderCtx, spec: &NodeParams) -> Result<Self, GetError> {
        match spec {
            NodeParams::WebRenderer { instance_id } => {
                let renderer = ctx.web_renderers.get(instance_id)?;
                Ok(Self::Web { renderer })
            }
            NodeParams::Shader {
                shader_id,
                shader_params,
                resolution,
            } => Ok(Self::Shader(ShaderNode::new(
                ctx.wgpu_ctx,
                ctx.shader_registry.get(shader_id)?,
                shader_params.as_ref(),
                None,
                *resolution,
            ))),
            NodeParams::Builtin {
                transformation,
                resolution,
            } => Ok(Self::Builtin(ShaderNode::new(
                ctx.wgpu_ctx,
                ctx.builtin_transforms.shader(transformation),
                BuiltinTransformations::params(transformation).as_ref(),
                BuiltinTransformations::clear_color(transformation),
                *resolution,
            ))),
            NodeParams::TextRenderer {
                text_params,
                resolution,
            } => {
                let renderer = TextRendererNode::new(ctx, text_params.clone(), resolution.clone());
                Ok(Self::Text(renderer))
            }
            NodeParams::Image { image_id } => {
                let node = ImageNode::new(ctx.image_registry.get(image_id)?);
                Ok(Self::Image(node))
            }
        }
    }

    pub fn render(
        &self,
        ctx: &mut RenderCtx,
        sources: &[(&NodeId, &NodeTexture)],
        target: &NodeTexture,
        pts: Duration,
    ) {
        match self {
            RenderNode::Shader(shader) => {
                shader.render(sources, target, pts);
            }
            RenderNode::Builtin(shader) => shader.render(sources, target, pts),
            RenderNode::Web { renderer } => renderer.render(ctx, sources, target),
            RenderNode::Text(renderer) => {
                renderer.render(ctx, target);
            }
            RenderNode::Image(node) => node.render(ctx, target, pts),
            RenderNode::InputStream => {
                // Nothing to do, textures on input nodes should be populated
                // at the start of render loop
            }
        }
    }

    pub fn resolution(&self) -> Option<Resolution> {
        match self {
            RenderNode::Shader(node) => Some(node.resolution()),
            RenderNode::Web { renderer } => Some(renderer.resolution()),
            RenderNode::Text(node) => Some(node.resolution()),
            RenderNode::Image(node) => Some(node.resolution()),
            RenderNode::InputStream => None,
            RenderNode::Builtin(node) => Some(node.resolution()),
        }
    }
}

pub struct Node {
    pub node_id: NodeId,
    pub output: NodeTexture,
    pub inputs: Vec<Arc<Node>>,
    pub renderer: RenderNode,
}

impl Node {
    pub fn new(ctx: &RenderCtx, spec: &NodeSpec, inputs: Vec<Arc<Node>>) -> Result<Self, GetError> {
        let node = RenderNode::new(ctx, &spec.params)?;
        let output = NodeTexture::new();
        if let Some(resolution) = node.resolution() {
            output.ensure_size(ctx.wgpu_ctx, resolution);
        }
        Ok(Self {
            node_id: spec.node_id.clone(),
            renderer: node,
            inputs,
            output,
        })
    }

    pub fn new_input(node_id: &NodeId) -> Result<Self, GetError> {
        let output = NodeTexture::new();

        Ok(Self {
            node_id: node_id.clone(),
            renderer: RenderNode::InputStream,
            inputs: vec![],
            output,
        })
    }
}

pub struct Scene {
    pub outputs: HashMap<OutputId, (Arc<Node>, OutputTexture)>,
    pub inputs: HashMap<InputId, (Arc<Node>, InputTexture)>,
}

#[derive(Debug, thiserror::Error)]
pub enum SceneUpdateError {
    #[error("Failed to construct render node")]
    RenderNodeError(#[source] GetError),

    #[error("Failed to construct input node")]
    InputNodeError(#[source] GetError),

    #[error("No spec for node with id {0}")]
    NoNodeWithIdError(NodeId),

    #[error("Scene definition is invalid")]
    InvalidSpec(#[from] SpecValidationError),

    #[error("Wgpu error")]
    WgpuError(#[from] WgpuError),

    #[error("Unknown resolution in the output node")]
    UnknownResolutionOnOutput(NodeId),
}

impl Scene {
    pub fn empty() -> Self {
        Self {
            outputs: HashMap::new(),
            inputs: HashMap::new(),
        }
    }

    pub fn update(&mut self, ctx: &RenderCtx, spec: &SceneSpec) -> Result<(), SceneUpdateError> {
        // TODO: If we want nodes to be stateful we could try reusing nodes instead
        //       of recreating them on every scene update
        let scope = WgpuErrorScope::push(&ctx.wgpu_ctx.device);

        let mut new_nodes = HashMap::new();
        let mut inputs = HashMap::new();
        let outputs = spec
            .outputs
            .iter()
            .map(|output| {
                let node =
                    Self::ensure_node(ctx, &output.input_pad, spec, &mut inputs, &mut new_nodes)?;
                let resolution = node.renderer.resolution().ok_or_else(|| {
                    SceneUpdateError::UnknownResolutionOnOutput(node.node_id.clone())
                })?;
                let buffers = OutputTexture::new(ctx.wgpu_ctx, resolution);
                Ok((output.output_id.clone(), (node, buffers)))
            })
            .collect::<Result<_, SceneUpdateError>>()?;

        scope.pop(&ctx.wgpu_ctx.device)?;

        self.inputs = inputs;
        self.outputs = outputs;

        Ok(())
    }

    fn ensure_node(
        ctx: &RenderCtx,
        node_id: &NodeId,
        spec: &SceneSpec,
        inputs: &mut HashMap<InputId, (Arc<Node>, InputTexture)>,
        new_nodes: &mut HashMap<NodeId, Arc<Node>>,
    ) -> Result<Arc<Node>, SceneUpdateError> {
        // check if node already exists
        if let Some(already_existing_node) = new_nodes.get(node_id) {
            return Ok(already_existing_node.clone());
        }

        // handle a case where node_id refers to transform node
        {
            let transform_spec = spec.nodes.iter().find(|node| &node.node_id == node_id);
            if let Some(transform) = transform_spec {
                let inputs = transform
                    .input_pads
                    .iter()
                    .map(|node_id| Self::ensure_node(ctx, node_id, spec, inputs, new_nodes))
                    .collect::<Result<_, _>>()?;
                let node =
                    Node::new(ctx, transform, inputs).map_err(SceneUpdateError::RenderNodeError)?;
                let node = Arc::new(node);
                new_nodes.insert(node_id.clone(), node.clone());
                return Ok(node);
            }
        }

        // If there is no node with id node_id, assume it's an input. Pipeline validation should
        // make sure that scene does not refer to missing entities.
        let node = Node::new_input(node_id).map_err(SceneUpdateError::InputNodeError)?;
        let node = Arc::new(node);
        new_nodes.insert(node_id.clone(), node.clone());
        inputs.insert(
            InputId(node_id.clone()),
            (node.clone(), InputTexture::new()),
        );
        Ok(node)
    }
}
