use std::sync::Arc;
use std::time::Duration;

use compositor_common::{
    scene::{InputId, OutputId, SceneSpec},
    Framerate,
};

use crate::wgpu::{WgpuCtx, WgpuErrorScope};
use crate::{
    error::{InitRendererEngineError, RenderSceneError, UpdateSceneError},
    transformations::{
        text_renderer::TextRendererCtx, web_renderer::chromium_context::ChromiumContext,
    },
    FrameSet, WebRendererOptions,
};

use self::{
    node::NodeSpecExt,
    render_graph::RenderGraph,
    render_loop::{populate_inputs, read_outputs, run_transforms},
    renderers::Renderers,
};

pub mod node;
pub mod render_graph;
mod render_loop;
pub mod renderers;

pub(crate) use render_loop::NodeRenderPass;

pub struct RendererOptions {
    pub web_renderer: WebRendererOptions,
    pub framerate: Framerate,
    pub stream_fallback_timeout: Duration,
}

pub struct Renderer {
    pub wgpu_ctx: Arc<WgpuCtx>,
    pub text_renderer_ctx: TextRendererCtx,
    pub chromium_context: Arc<ChromiumContext>,

    pub render_graph: RenderGraph,
    pub scene_spec: Arc<SceneSpec>,

    pub(crate) renderers: Renderers,

    stream_fallback_timeout: Duration,
}

pub struct RenderCtx<'a> {
    pub wgpu_ctx: &'a Arc<WgpuCtx>,

    pub text_renderer_ctx: &'a TextRendererCtx,
    pub chromium: &'a Arc<ChromiumContext>,

    pub(crate) renderers: &'a Renderers,

    pub(crate) stream_fallback_timeout: Duration,
}

pub struct RegisterCtx {
    pub wgpu_ctx: Arc<WgpuCtx>,
    pub chromium: Arc<ChromiumContext>,
}

impl Renderer {
    pub fn new(opts: RendererOptions) -> Result<Self, InitRendererEngineError> {
        let wgpu_ctx = Arc::new(WgpuCtx::new()?);

        Ok(Self {
            wgpu_ctx: wgpu_ctx.clone(),
            text_renderer_ctx: TextRendererCtx::new(),
            chromium_context: Arc::new(ChromiumContext::new(opts.web_renderer, opts.framerate)?),
            render_graph: RenderGraph::empty(),
            renderers: Renderers::new(wgpu_ctx)?,
            scene_spec: Arc::new(SceneSpec {
                nodes: vec![],
                outputs: vec![],
            }),

            stream_fallback_timeout: opts.stream_fallback_timeout,
        })
    }

    pub(super) fn register_ctx(&self) -> RegisterCtx {
        RegisterCtx {
            wgpu_ctx: self.wgpu_ctx.clone(),
            chromium: self.chromium_context.clone(),
        }
    }

    pub fn render(
        &mut self,
        mut inputs: FrameSet<InputId>,
    ) -> Result<FrameSet<OutputId>, RenderSceneError> {
        let ctx = &mut RenderCtx {
            wgpu_ctx: &self.wgpu_ctx,
            chromium: &self.chromium_context,
            text_renderer_ctx: &self.text_renderer_ctx,
            renderers: &self.renderers,
            stream_fallback_timeout: self.stream_fallback_timeout,
        };

        let scope = WgpuErrorScope::push(&ctx.wgpu_ctx.device);

        populate_inputs(ctx, &mut self.render_graph, &mut inputs).unwrap();
        run_transforms(ctx, &mut self.render_graph, inputs.pts).unwrap();
        let frames = read_outputs(ctx, &mut self.render_graph, inputs.pts).unwrap();

        scope.pop(&ctx.wgpu_ctx.device)?;

        Ok(FrameSet {
            frames,
            pts: inputs.pts,
        })
    }

    pub fn update_scene(&mut self, scene_spec: Arc<SceneSpec>) -> Result<(), UpdateSceneError> {
        self.validate_constraints(&scene_spec)?;
        self.render_graph.update(
            &RenderCtx {
                wgpu_ctx: &self.wgpu_ctx,
                text_renderer_ctx: &self.text_renderer_ctx,
                chromium: &self.chromium_context,
                renderers: &self.renderers,
                stream_fallback_timeout: self.stream_fallback_timeout,
            },
            &scene_spec,
        )?;
        self.scene_spec = scene_spec;
        Ok(())
    }

    fn validate_constraints(&self, scene_spec: &SceneSpec) -> Result<(), UpdateSceneError> {
        for node_spec in &scene_spec.nodes {
            node_spec
                .constraints(&self.renderers)?
                .check(scene_spec, &node_spec.node_id)
                .map_err(|err| {
                    UpdateSceneError::ConstraintsValidationError(err, node_spec.node_id.clone())
                })?;
        }

        Ok(())
    }
}
