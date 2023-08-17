use std::{collections::HashMap, sync::Arc, time::Duration};

use compositor_common::{
    scene::{
        InputId, InputSpec, NodeId, OutputId, Resolution, SceneSpec, TransformNodeSpec,
        TransformParams,
    },
    SpecValidationError,
};
use log::error;

use crate::{
    registry::GetError,
    transformations::{
        image_renderer::ImageNode, shader::node::ShaderNode, text_renderer::TextRendererNode,
        web_renderer::WebRenderer,
    },
};

use super::{
    texture::{InputTexture, NodeTexture, OutputTexture},
    RenderCtx,
};

pub struct InputNode {}

pub enum TransformNode {
    Shader(ShaderNode),
    WebRenderer { renderer: Arc<WebRenderer> },
    TextRenderer(TextRendererNode),
    ImageRenderer(ImageNode),
    Nop,
}

impl TransformNode {
    fn new(ctx: &RenderCtx, spec: &TransformParams) -> Result<(Self, Resolution), GetError> {
        match spec {
            TransformParams::WebRenderer { renderer_id } => {
                let renderer = ctx.web_renderers.get(renderer_id)?;
                let resolution = renderer.resolution();
                Ok((TransformNode::WebRenderer { renderer }, resolution))
            }
            TransformParams::Shader {
                shader_id,
                shader_params,
                resolution,
            } => Ok((
                TransformNode::Shader(ShaderNode::new(
                    ctx.wgpu_ctx,
                    ctx.shader_transforms.get(shader_id)?,
                    shader_params.as_ref(),
                )),
                *resolution,
            )),
            TransformParams::TextRenderer {
                text_params,
                resolution,
            } => {
                let (renderer, resolution) =
                    TextRendererNode::new(ctx, text_params.clone(), resolution.clone());
                Ok((TransformNode::TextRenderer(renderer), resolution))
            }
            TransformParams::Image { image_id } => {
                let node = ImageNode::new(ctx.image_registry.get(image_id)?);
                let resolution = node.resolution();
                Ok((TransformNode::ImageRenderer(node), resolution))
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
            TransformNode::Shader(shader) => {
                shader.render(sources, target, pts);
            }
            TransformNode::WebRenderer { renderer } => {
                renderer.render(ctx, sources, target);
            }
            TransformNode::TextRenderer(renderer) => {
                renderer.render(ctx, target);
            }
            TransformNode::ImageRenderer(node) => node.render(ctx, target),
            TransformNode::Nop => (),
        }
    }
}

pub struct Node {
    pub node_id: NodeId,
    pub output: NodeTexture,
    pub resolution: Resolution,
    pub inputs: Vec<Arc<Node>>,
    pub transform: TransformNode,
}

impl Node {
    pub fn new(
        ctx: &RenderCtx,
        spec: &TransformNodeSpec,
        inputs: Vec<Arc<Node>>,
    ) -> Result<Self, GetError> {
        let (node, resolution) = TransformNode::new(ctx, &spec.transform_params)?;
        let output = NodeTexture::new(ctx.wgpu_ctx, resolution);
        Ok(Self {
            node_id: spec.node_id.clone(),
            transform: node,
            resolution,
            inputs,
            output,
        })
    }

    pub fn new_input(ctx: &RenderCtx, spec: &InputSpec) -> Result<Self, GetError> {
        let output = NodeTexture::new(ctx.wgpu_ctx, spec.resolution);

        Ok(Self {
            node_id: spec.input_id.0.clone(),
            transform: TransformNode::Nop,
            resolution: spec.resolution,
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
    #[error("Failed to construct transform node")]
    TransformNodeError(#[source] GetError),

    #[error("Failed to construct input node")]
    InputNodeError(#[source] GetError),

    #[error("No spec for node with id {0}")]
    NoNodeWithIdError(NodeId),

    #[error("Scene definition is invalid")]
    InvalidSpec(#[source] SpecValidationError),
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
        let mut new_nodes = HashMap::new();
        self.inputs = HashMap::new();
        self.outputs = spec
            .outputs
            .iter()
            .map(|output| {
                let node = self.ensure_node(ctx, &output.input_pad, spec, &mut new_nodes)?;
                let buffers = OutputTexture::new(ctx.wgpu_ctx, node.resolution);
                Ok((output.output_id.clone(), (node, buffers)))
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
        if let Some(already_existing_node) = new_nodes.get(node_id) {
            return Ok(already_existing_node.clone());
        }

        // handle a case where node_id refers to transform node
        {
            let transform_spec = spec.transforms.iter().find(|node| &node.node_id == node_id);
            if let Some(transform) = transform_spec {
                let inputs = transform
                    .input_pads
                    .iter()
                    .map(|node_id| self.ensure_node(ctx, node_id, spec, new_nodes))
                    .collect::<Result<_, _>>()?;
                let node = Node::new(ctx, transform, inputs)
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
                self.inputs.insert(
                    InputId(node_id.clone()),
                    (
                        node.clone(),
                        InputTexture::new(ctx.wgpu_ctx, input.resolution, input.fallback_color_rgb),
                    ),
                );
                return Ok(node);
            }
        }

        Err(SceneUpdateError::NoNodeWithIdError(node_id.clone()))
    }
}
