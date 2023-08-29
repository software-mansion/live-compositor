use std::{collections::HashMap, sync::Arc, time::Duration};

use compositor_common::{
    scene::{InputId, NodeId, NodeParams, NodeSpec, OutputId, Resolution, SceneSpec},
    SpecValidationError,
};
use log::error;

use crate::{
    registry::GetError,
    render_loop::NodeRenderPass,
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
                let renderer = ctx.transformations.web_renderers.get(instance_id)?;
                Ok(Self::Web { renderer })
            }
            NodeParams::Shader {
                shader_id,
                shader_params,
                resolution,
            } => Ok(Self::Shader(ShaderNode::new(
                ctx.wgpu_ctx,
                ctx.transformations.shaders.get(shader_id)?,
                shader_params.as_ref(),
                None,
                *resolution,
            ))),
            NodeParams::Builtin {
                transformation,
                resolution,
            } => Ok(Self::Builtin(ShaderNode::new(
                ctx.wgpu_ctx,
                ctx.transformations.builtin.shader(transformation),
                BuiltinTransformations::params(transformation, resolution).as_ref(),
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
                let node = ImageNode::new(ctx.transformations.images.get(image_id)?);
                Ok(Self::Image(node))
            }
        }
    }

    pub fn render(
        &self,
        ctx: &mut RenderCtx,
        sources: &[(&NodeId, &NodeTexture)],
        target: &mut NodeTexture,
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
    pub inputs: Vec<NodeId>,
    pub renderer: RenderNode,
}

impl Node {
    pub fn new(ctx: &RenderCtx, spec: &NodeSpec) -> Result<Self, GetError> {
        let node = RenderNode::new(ctx, &spec.params)?;
        let mut output = NodeTexture::new();
        if let Some(resolution) = node.resolution() {
            output.ensure_size(ctx.wgpu_ctx, resolution);
        }
        Ok(Self {
            node_id: spec.node_id.clone(),
            renderer: node,
            inputs: spec.input_pads.clone(),
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
    pub nodes: SceneNodesSet,
    pub outputs: HashMap<OutputId, (NodeId, OutputTexture)>,
    pub inputs: HashMap<InputId, InputTexture>,
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
            nodes: SceneNodesSet::new(),
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
                Self::ensure_node(ctx, &output.input_pad, spec, &mut inputs, &mut new_nodes)?;
                let node = new_nodes
                    .get(&output.input_pad)
                    .ok_or_else(|| SceneUpdateError::NoNodeWithIdError(output.input_pad.clone()))?;
                let resolution = node.renderer.resolution().ok_or_else(|| {
                    SceneUpdateError::UnknownResolutionOnOutput(node.node_id.clone())
                })?;
                let output_texture = OutputTexture::new(ctx.wgpu_ctx, resolution);
                Ok((
                    output.output_id.clone(),
                    (node.node_id.clone(), output_texture),
                ))
            })
            .collect::<Result<_, SceneUpdateError>>()?;

        scope.pop(&ctx.wgpu_ctx.device)?;

        self.inputs = inputs;
        self.outputs = outputs;
        self.nodes = SceneNodesSet { nodes: new_nodes };

        Ok(())
    }

    fn ensure_node(
        ctx: &RenderCtx,
        node_id: &NodeId,
        spec: &SceneSpec,
        inputs: &mut HashMap<InputId, InputTexture>,
        new_nodes: &mut HashMap<NodeId, Node>,
    ) -> Result<(), SceneUpdateError> {
        // check if node already exists
        if new_nodes.get(node_id).is_some() {
            return Ok(());
        }

        // handle a case where node_id refers to transform node
        {
            let node_spec = spec.nodes.iter().find(|node| &node.node_id == node_id);
            if let Some(node_spec) = node_spec {
                for child_id in &node_spec.input_pads {
                    Self::ensure_node(ctx, child_id, spec, inputs, new_nodes)?;
                }
                let node = Node::new(ctx, node_spec).map_err(SceneUpdateError::RenderNodeError)?;
                new_nodes.insert(node_id.clone(), node);
                return Ok(());
            }
        }

        // If there is no node with id node_id, assume it's an input. Pipeline validation should
        // make sure that scene does not refer to missing entities.
        let node = Node::new_input(node_id).map_err(SceneUpdateError::InputNodeError)?;
        new_nodes.insert(node_id.clone(), node);
        inputs.insert(InputId(node_id.clone()), InputTexture::new());
        Ok(())
    }
}

#[derive(Default)]
pub struct SceneNodesSet {
    nodes: HashMap<NodeId, Node>,
}

impl SceneNodesSet {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    pub fn node(&self, node_id: &NodeId) -> Result<&Node, SceneError> {
        self.nodes
            .get(node_id)
            .ok_or_else(|| SceneError::MissingNode(node_id.clone()))
    }

    pub fn node_mut(&mut self, node_id: &NodeId) -> Result<&mut Node, SceneError> {
        self.nodes
            .get_mut(node_id)
            .ok_or_else(|| SceneError::MissingNode(node_id.clone()))
    }

    /// Borrow all nodes that are needed to render node node_id.
    pub fn node_render_pass<'a>(
        &'a mut self,
        node_id: &NodeId,
    ) -> Result<NodeRenderPass<'a>, SceneError> {
        let input_ids: Vec<NodeId> = self.node(node_id)?.inputs.to_vec();

        // Borrow all the references we will need for a single node render.
        let mut node_and_inputs: HashMap<&NodeId, &mut Node> = self
            .nodes
            .iter_mut()
            .filter(|(id, _node)| input_ids.contains(id) || *id == node_id)
            .collect();

        // Extract mutable borrow for the node we will render.
        let node = node_and_inputs
            .remove(&node_id)
            .ok_or_else(|| SceneError::MissingNode(node_id.clone()))?;

        // Convert mutable borrows on rest of the nodes into immutable.
        // One input might be used multiple times, so we might need to
        // borrow it more than once, so it needs to be immutable.
        let inputs_map: HashMap<&NodeId, &Node> = node_and_inputs
            .into_iter()
            .map(|(id, node)| (id, &*node))
            .collect();

        // Get immutable borrows for inputs.
        let inputs: Vec<&Node> = input_ids
            .into_iter()
            .map(|input_id| {
                inputs_map
                    .get(&input_id)
                    .copied()
                    .ok_or_else(|| SceneError::MissingNode(node_id.clone()))
            })
            .collect::<Result<Vec<_>, SceneError>>()?;
        Ok(NodeRenderPass { node, inputs })
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SceneError {
    #[error("Missing node with id {0}")]
    MissingNode(NodeId),
}
