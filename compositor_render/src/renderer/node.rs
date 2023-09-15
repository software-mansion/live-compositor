use std::time::Duration;

use compositor_common::renderer_spec::{FallbackStrategy, RendererId};

use compositor_common::scene::{NodeId, NodeParams, NodeSpec, Resolution};
use log::error;

use crate::gpu_shader::error::ParametersValidationError;
use crate::transformations::builtin::Builtin;
use crate::transformations::shader::node::ShaderNode;
use crate::transformations::{
    builtin::node::BuiltinNode, image_renderer::ImageNode, text_renderer::TextRendererNode,
    web_renderer::node::WebRendererNode,
};

use super::{texture::NodeTexture, RenderCtx};

pub enum RenderNode {
    Shader(ShaderNode),
    Web(WebRendererNode),
    Text(TextRendererNode),
    Image(ImageNode),
    Builtin(BuiltinNode),
    InputStream,
}

impl RenderNode {
    fn new(ctx: &RenderCtx, spec: &NodeSpec) -> Result<Self, CreateNodeError> {
        match &spec.params {
            NodeParams::WebRenderer { instance_id } => {
                let renderer = ctx
                    .renderers
                    .web_renderers
                    .get(instance_id)
                    .ok_or_else(|| CreateNodeError::WebRendererNotFound(instance_id.clone()))?;
                Ok(Self::Web(WebRendererNode::new(renderer)))
            }
            NodeParams::Shader {
                shader_id,
                shader_params,
                resolution,
            } => {
                let shader = ctx
                    .renderers
                    .shaders
                    .get(shader_id)
                    .ok_or_else(|| CreateNodeError::ShaderNotFound(shader_id.clone()))?;

                let node =
                    ShaderNode::new(ctx.wgpu_ctx, shader, shader_params.as_ref(), *resolution)?;
                Ok(Self::Shader(node))
            }
            NodeParams::Builtin { transformation } => {
                let gpu_shader = ctx.renderers.builtin.gpu_shader(transformation);
                let input_count = spec.input_pads.len() as u32;

                Ok(Self::Builtin(BuiltinNode::new(
                    Builtin {
                        spec: transformation.clone(),
                        gpu_shader,
                    },
                    input_count,
                )))
            }
            NodeParams::Text(text_spec) => {
                let renderer = TextRendererNode::new(ctx, text_spec.clone());
                Ok(Self::Text(renderer))
            }
            NodeParams::Image { image_id } => {
                let image = ctx
                    .renderers
                    .images
                    .get(image_id)
                    .ok_or_else(|| CreateNodeError::ImageNotFound(image_id.clone()))?;
                let node = ImageNode::new(image);
                Ok(Self::Image(node))
            }
        }
    }

    pub fn render(
        &mut self,
        ctx: &mut RenderCtx,
        sources: &[(&NodeId, &NodeTexture)],
        target: &mut NodeTexture,
        pts: Duration,
    ) {
        if self.should_fallback(sources) {
            target.clear();
            return;
        }

        match self {
            RenderNode::Shader(ref shader) => {
                shader.render(sources, target, pts);
            }
            RenderNode::Builtin(builtin_node) => builtin_node.render(sources, target, pts),
            RenderNode::Web(renderer) => renderer.render(ctx, sources, target),
            RenderNode::Text(ref renderer) => {
                renderer.render(ctx, target);
            }
            RenderNode::Image(ref node) => node.render(ctx, target, pts),
            RenderNode::InputStream => {
                // Nothing to do, textures on input nodes should be populated
                // at the start of render loop
            }
        }
    }

    pub fn resolution(&self) -> Option<Resolution> {
        match self {
            RenderNode::Shader(node) => Some(node.resolution()),
            RenderNode::Web(node) => Some(node.resolution()),
            RenderNode::Text(node) => Some(node.resolution()),
            RenderNode::Image(node) => Some(node.resolution()),
            RenderNode::InputStream => None,
            RenderNode::Builtin(node) => node.resolution_from_spec(),
        }
    }

    fn should_fallback(&self, sources: &[(&NodeId, &NodeTexture)]) -> bool {
        if sources.is_empty() {
            return false;
        }

        match self.fallback_strategy() {
            FallbackStrategy::NeverFallback => false,
            FallbackStrategy::FallbackIfAllInputsMissing => sources
                .iter()
                .all(|(_, node_texture)| node_texture.is_empty()),
            FallbackStrategy::FallbackIfAnyInputMissing => sources
                .iter()
                .any(|(_, node_texture)| node_texture.is_empty()),
        }
    }

    fn fallback_strategy(&self) -> FallbackStrategy {
        match self {
            RenderNode::Shader(shader_node) => shader_node.fallback_strategy(),
            RenderNode::Web(web_renderer_node) => web_renderer_node.fallback_strategy(),
            RenderNode::Text(_) => FallbackStrategy::NeverFallback,
            RenderNode::Image(_) => FallbackStrategy::NeverFallback,
            RenderNode::Builtin(builtin_node) => builtin_node.fallback_strategy(),
            RenderNode::InputStream => FallbackStrategy::NeverFallback,
        }
    }
}

pub struct Node {
    pub node_id: NodeId,
    pub output: NodeTexture,
    pub inputs: Vec<NodeId>,
    pub fallback: Option<NodeId>,
    pub renderer: RenderNode,
}

impl Node {
    pub fn new(ctx: &RenderCtx, spec: &NodeSpec) -> Result<Self, CreateNodeError> {
        let node = RenderNode::new(ctx, spec)?;
        let mut output = NodeTexture::new();
        if let Some(resolution) = node.resolution() {
            output.ensure_size(ctx.wgpu_ctx, resolution);
        }

        Ok(Self {
            node_id: spec.node_id.clone(),
            renderer: node,
            inputs: spec.input_pads.clone(),
            fallback: spec.fallback_id.clone(),
            output,
        })
    }

    pub fn new_input(node_id: &NodeId) -> Self {
        let output = NodeTexture::new();

        Self {
            node_id: node_id.clone(),
            renderer: RenderNode::InputStream,
            inputs: vec![],
            fallback: None,
            output,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CreateNodeError {
    #[error("Shader \"{0}\" does not exist. You have to register it first before using it in the scene definition.")]
    ShaderNotFound(RendererId),

    #[error("Error while validating the parameters for the shader node:\n{0}")]
    ShaderNodeParametersValidationError(#[from] ParametersValidationError),

    #[error("Instance of web renderer \"{0}\" does not exist. You have to register it first before using it in the scene definition.")]
    WebRendererNotFound(RendererId),

    #[error("Image \"{0}\" does not exist. You have to register it first before using it in the scene definition.")]
    ImageNotFound(RendererId),
}
