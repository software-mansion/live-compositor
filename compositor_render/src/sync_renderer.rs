use std::sync::{Arc, Mutex};

use compositor_common::{
    renderer_spec::RendererSpec,
    scene::{InputId, OutputId, SceneSpec},
};

use crate::{
    error::{InitRendererEngineError, RegisterRendererError, RenderSceneError},
    event_loop::EventLoop,
    frame_set::FrameSet,
    renderer::{scene::UpdateSceneError, Renderer, RendererOptions},
    transformations::{image_renderer::Image, shader::Shader, web_renderer::WebRenderer},
};

#[derive(Clone)]
pub struct SyncRenderer(Arc<Mutex<Renderer>>);

impl SyncRenderer {
    pub fn new(opts: RendererOptions) -> Result<(Self, EventLoop), InitRendererEngineError> {
        let renderer = Renderer::new(opts)?;
        let event_loop = EventLoop::new(renderer.chromium_context.cef_context());

        Ok((Self(Arc::new(Mutex::new(renderer))), event_loop))
    }

    pub fn register_renderer(&self, spec: RendererSpec) -> Result<(), RegisterRendererError> {
        let ctx = self.0.lock().unwrap().register_ctx();
        let mut guard = self.0.lock().unwrap();
        match spec {
            RendererSpec::Shader(spec) => {
                let shader = Arc::new(Shader::new(
                    &ctx.wgpu_ctx,
                    spec.source,
                    spec.fallback_strategy,
                )?);

                Ok(guard.renderers.shaders.register(spec.shader_id, shader)?)
            }
            RendererSpec::WebRenderer(params) => {
                let instance_id = params.instance_id.clone();
                let web = Arc::new(WebRenderer::new(&ctx, params));

                Ok(guard.renderers.web_renderers.register(instance_id, web)?)
            }
            RendererSpec::Image(spec) => {
                let image_id = spec.image_id.clone();
                let asset = Image::new(&ctx, spec)?;

                Ok(guard.renderers.images.register(image_id, asset)?)
            }
        }
    }

    pub fn render(&self, input: FrameSet<InputId>) -> Result<FrameSet<OutputId>, RenderSceneError> {
        self.0.lock().unwrap().render(input)
    }

    pub fn update_scene(&mut self, scene_specs: Arc<SceneSpec>) -> Result<(), UpdateSceneError> {
        self.0.lock().unwrap().update_scene(scene_specs)
    }

    pub fn scene_spec(&self) -> Arc<SceneSpec> {
        self.0.lock().unwrap().scene_spec.clone()
    }
}
