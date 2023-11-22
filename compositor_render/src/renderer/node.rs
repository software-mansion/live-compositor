use std::sync::Arc;
use std::time::Duration;

use compositor_common::renderer_spec::FallbackStrategy;

use compositor_common::scene::constraints::NodeConstraints;
use compositor_common::scene::{NodeParams, NodeSpec};

use crate::error::{CreateNodeError, UpdateSceneError};

use crate::scene::{self, ShaderComponentParams};
use crate::transformations::image_renderer::Image;
use crate::transformations::layout::LayoutNode;
use crate::transformations::shader::node::ShaderNode;
use crate::transformations::shader::Shader;

use crate::transformations::text_renderer::TextRenderParams;
use crate::transformations::web_renderer::WebRenderer;
use crate::transformations::{
    image_renderer::ImageNode, text_renderer::TextRendererNode, web_renderer::node::WebRendererNode,
};
use crate::wgpu::texture::NodeTexture;

use super::render_graph::NodeId;
use super::renderers::Renderers;
use super::RenderCtx;

pub(crate) enum InnerRenderNode {
    Shader(ShaderNode),
    #[allow(dead_code)]
    Web(WebRendererNode),
    #[allow(dead_code)]
    Text(TextRendererNode),
    #[allow(dead_code)]
    Image(ImageNode),
    #[allow(dead_code)]
    Layout(LayoutNode),
    InputStream,
}

impl InnerRenderNode {
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
            InnerRenderNode::Shader(ref shader) => {
                shader.render(sources, target, pts);
            }
            InnerRenderNode::Web(renderer) => renderer.render(ctx, sources, target),
            InnerRenderNode::Text(renderer) => {
                renderer.render(ctx, target);
            }
            InnerRenderNode::Image(ref node) => node.render(ctx, target, pts),
            InnerRenderNode::InputStream => {
                // Nothing to do, textures on input nodes should be populated
                // at the start of render loop
            }
            InnerRenderNode::Layout(node) => node.render(ctx, sources, target, pts),
        }
    }

    // TODO: move to FallbackStrategyExt
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
            InnerRenderNode::Shader(shader_node) => shader_node.fallback_strategy(),
            InnerRenderNode::Web(web_renderer_node) => web_renderer_node.fallback_strategy(),
            InnerRenderNode::Text(_) => FallbackStrategy::NeverFallback,
            InnerRenderNode::Image(_) => FallbackStrategy::NeverFallback,
            InnerRenderNode::InputStream => FallbackStrategy::NeverFallback,
            InnerRenderNode::Layout(_) => FallbackStrategy::NeverFallback,
        }
    }
}

pub struct RenderNode {
    pub(crate) output: NodeTexture,
    pub(crate) inputs: Vec<NodeId>,
    pub(crate) fallback: Option<NodeId>,
    pub(crate) renderer: InnerRenderNode,
}

impl RenderNode {
    pub(super) fn new_shader_node(
        ctx: &RenderCtx,
        inputs: Vec<NodeId>,
        shader_params: ShaderComponentParams,
        shader: Arc<Shader>,
    ) -> Result<Self, CreateNodeError> {
        let node = InnerRenderNode::Shader(ShaderNode::new(
            ctx,
            shader,
            &shader_params.shader_id,
            &shader_params.shader_param,
            &shader_params.size.into(),
        )?);
        let mut output = NodeTexture::new();
        output.ensure_size(ctx.wgpu_ctx, shader_params.size.into());

        Ok(Self {
            renderer: node,
            inputs,
            fallback: None,
            output,
        })
    }

    pub(super) fn new_web_renderer_node(
        ctx: &RenderCtx,
        inputs: Vec<NodeId>,
        node_id: &NodeId,
        web_renderer: Arc<WebRenderer>,
    ) -> Self {
        let resolution = web_renderer.resolution();
        let node = InnerRenderNode::Web(WebRendererNode::new(node_id, web_renderer));
        let mut output = NodeTexture::new();
        output.ensure_size(ctx.wgpu_ctx, resolution);

        Self {
            renderer: node,
            inputs,
            fallback: None,
            output,
        }
    }

    pub(super) fn new_image_node(image: Image) -> Self {
        let node = InnerRenderNode::Image(ImageNode::new(image));
        let output = NodeTexture::new();

        Self {
            renderer: node,
            inputs: vec![],
            fallback: None,
            output,
        }
    }

    pub(super) fn new_text_node(params: TextRenderParams) -> Self {
        let node = InnerRenderNode::Text(TextRendererNode::new(params));
        let output = NodeTexture::new();

        Self {
            renderer: node,
            inputs: vec![],
            fallback: None,
            output,
        }
    }

    pub(super) fn new_layout_node(
        ctx: &RenderCtx,
        inputs: Vec<NodeId>,
        provider: scene::LayoutNode,
    ) -> Result<Self, CreateNodeError> {
        let node = InnerRenderNode::Layout(LayoutNode::new(ctx, Box::new(provider)));
        let output = NodeTexture::new();

        Ok(Self {
            renderer: node,
            inputs,
            fallback: None,
            output,
        })
    }

    pub(crate) fn new_input() -> Self {
        let output = NodeTexture::new();

        Self {
            renderer: InnerRenderNode::InputStream,
            inputs: vec![],
            fallback: None,
            output,
        }
    }
}

pub(crate) trait NodeSpecExt {
    fn constraints<'a>(
        &self,
        renderers: &'a Renderers,
    ) -> Result<&'a NodeConstraints, UpdateSceneError>;
}

impl NodeSpecExt for NodeSpec {
    fn constraints<'a>(
        &self,
        renderers: &'a Renderers,
    ) -> Result<&'a NodeConstraints, UpdateSceneError> {
        match &self.params {
            NodeParams::WebRenderer { instance_id } => renderers
                .web_renderers
                .get_ref(instance_id)
                .map(|web_renderer| web_renderer.constraints())
                .ok_or_else(|| {
                    UpdateSceneError::CreateNodeError(
                        crate::error::CreateNodeError::WebRendererNotFound(instance_id.clone()),
                        0,
                    )
                }),
            NodeParams::Shader { shader_id, .. } => renderers
                .shaders
                .get_ref(shader_id)
                .map(|shader| shader.constraints())
                .ok_or_else(|| {
                    UpdateSceneError::CreateNodeError(
                        crate::error::CreateNodeError::ShaderNotFound(shader_id.clone()),
                        0, //TODO
                    )
                }),
            NodeParams::Image { .. } => Ok(NodeParams::image_constraints()),
        }
    }
}
