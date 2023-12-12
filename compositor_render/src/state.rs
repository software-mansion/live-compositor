use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::error::{RegisterRendererError, UnregisterRendererError};
use crate::image;
use crate::scene::OutputScene;
use crate::transformations::image_renderer::Image;
use crate::transformations::shader::Shader;
use crate::transformations::web_renderer::WebRenderer;
use crate::{
    error::{InitRendererEngineError, RenderSceneError, UpdateSceneError},
    transformations::{
        text_renderer::TextRendererCtx, web_renderer::chromium_context::ChromiumContext,
    },
    types::Framerate,
    EventLoop, FrameSet, InputId, OutputId,
};
use crate::{
    scene::{self, SceneState},
    wgpu::{WgpuCtx, WgpuErrorScope},
};
use crate::{shader, web_renderer, RegistryType, RendererId};

use self::{
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
    pub web_renderer: web_renderer::WebRendererInitOptions,
    pub framerate: Framerate,
    pub stream_fallback_timeout: Duration,
}

#[derive(Clone)]
pub struct Renderer(Arc<Mutex<InnerRenderer>>);

struct InnerRenderer {
    wgpu_ctx: Arc<WgpuCtx>,
    text_renderer_ctx: TextRendererCtx,
    chromium_context: Arc<ChromiumContext>,

    render_graph: RenderGraph,
    scene: SceneState,

    renderers: Renderers,

    stream_fallback_timeout: Duration,
}

pub(crate) struct RenderCtx<'a> {
    pub(crate) wgpu_ctx: &'a Arc<WgpuCtx>,
    pub(crate) text_renderer_ctx: &'a TextRendererCtx,
    pub(crate) renderers: &'a Renderers,
    pub(crate) stream_fallback_timeout: Duration,
}

pub(crate) struct RegisterCtx {
    pub(crate) wgpu_ctx: Arc<WgpuCtx>,
    pub(crate) chromium: Arc<ChromiumContext>,
}

/// RendererSpec provides configuration necessary to construct Renderer. Renderers
/// are entities like shader, image or chromium_instance and can be used by nodes
/// to transform or generate frames.
#[derive(Debug)]
pub enum RendererSpec {
    Shader(shader::ShaderSpec),
    WebRenderer(web_renderer::WebRendererSpec),
    Image(image::ImageSpec),
}

impl Renderer {
    pub fn new(opts: RendererOptions) -> Result<(Self, EventLoop), InitRendererEngineError> {
        let renderer = InnerRenderer::new(opts)?;
        let event_loop = EventLoop::new(renderer.chromium_context.cef_context());

        Ok((Self(Arc::new(Mutex::new(renderer))), event_loop))
    }

    pub fn register_renderer(&self, spec: RendererSpec) -> Result<(), RegisterRendererError> {
        let ctx = self.0.lock().unwrap().register_ctx();
        match spec {
            RendererSpec::Shader(spec) => {
                let shader_id = spec.shader_id.clone();

                let shader = Shader::new(&ctx.wgpu_ctx, spec)
                    .map_err(|err| RegisterRendererError::Shader(err, shader_id.clone()))?;

                let mut guard = self.0.lock().unwrap();
                Ok(guard
                    .renderers
                    .shaders
                    .register(shader_id, Arc::new(shader))?)
            }
            RendererSpec::WebRenderer(params) => {
                let instance_id = params.instance_id.clone();
                let web = WebRenderer::new(&ctx, params)
                    .map_err(|err| RegisterRendererError::Web(err, instance_id.clone()))?;

                let mut guard = self.0.lock().unwrap();
                Ok(guard
                    .renderers
                    .web_renderers
                    .register(instance_id, Arc::new(web))?)
            }
            RendererSpec::Image(spec) => {
                let image_id = spec.image_id.clone();
                let asset = Image::new(&ctx, spec)
                    .map_err(|err| RegisterRendererError::Image(err, image_id.clone()))?;

                let mut guard = self.0.lock().unwrap();
                Ok(guard.renderers.images.register(image_id, asset)?)
            }
        }
    }

    pub fn unregister_renderer(
        &self,
        renderer_id: &RendererId,
        registry_type: RegistryType,
    ) -> Result<(), UnregisterRendererError> {
        let mut guard = self.0.lock().unwrap();
        match registry_type {
            RegistryType::Shader => guard.renderers.shaders.unregister(renderer_id)?,
            RegistryType::WebRenderer => guard.renderers.web_renderers.unregister(renderer_id)?,
            RegistryType::Image => guard.renderers.images.unregister(renderer_id)?,
        }
        Ok(())
    }

    pub fn render(&self, input: FrameSet<InputId>) -> Result<FrameSet<OutputId>, RenderSceneError> {
        self.0.lock().unwrap().render(input)
    }

    pub fn update_scene(&mut self, scene_specs: Vec<OutputScene>) -> Result<(), UpdateSceneError> {
        self.0.lock().unwrap().update_scene(scene_specs)
    }
}

impl InnerRenderer {
    pub fn new(opts: RendererOptions) -> Result<Self, InitRendererEngineError> {
        let wgpu_ctx = Arc::new(WgpuCtx::new()?);

        Ok(Self {
            wgpu_ctx: wgpu_ctx.clone(),
            text_renderer_ctx: TextRendererCtx::new(),
            chromium_context: Arc::new(ChromiumContext::new(opts.web_renderer, opts.framerate)?),
            render_graph: RenderGraph::empty(),
            renderers: Renderers::new(wgpu_ctx)?,
            stream_fallback_timeout: opts.stream_fallback_timeout,
            scene: SceneState::new(),
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
            text_renderer_ctx: &self.text_renderer_ctx,
            renderers: &self.renderers,
            stream_fallback_timeout: self.stream_fallback_timeout,
        };

        let scope = WgpuErrorScope::push(&ctx.wgpu_ctx.device);

        let input_resolutions = inputs
            .frames
            .iter()
            .map(|(input_id, frame)| (input_id.clone(), frame.resolution))
            .collect();
        self.scene
            .register_render_event(inputs.pts, input_resolutions);

        populate_inputs(ctx, &mut self.render_graph, &mut inputs).unwrap();
        run_transforms(ctx, &mut self.render_graph, inputs.pts).unwrap();
        let frames = read_outputs(ctx, &mut self.render_graph, inputs.pts).unwrap();

        scope.pop(&ctx.wgpu_ctx.device)?;

        Ok(FrameSet {
            frames,
            pts: inputs.pts,
        })
    }

    pub fn update_scene(
        &mut self,
        scenes: Vec<scene::OutputScene>,
    ) -> Result<(), UpdateSceneError> {
        let output_nodes =
            self.scene
                .update_scene(scenes, &self.renderers, &self.text_renderer_ctx)?;
        self.render_graph.update(
            &RenderCtx {
                wgpu_ctx: &self.wgpu_ctx,
                text_renderer_ctx: &self.text_renderer_ctx,
                renderers: &self.renderers,
                stream_fallback_timeout: self.stream_fallback_timeout,
            },
            output_nodes,
        )?;
        Ok(())
    }
}
