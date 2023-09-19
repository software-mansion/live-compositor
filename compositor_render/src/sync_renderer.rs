use std::sync::{Arc, Mutex};

use compositor_common::{
    renderer_spec::{RendererId, RendererSpec},
    scene::{InputId, OutputId, SceneSpec},
};

use crate::{
    error::{
        InitRendererEngineError, RegisterRendererError, RenderSceneError, UnregisterRendererError,
        UpdateSceneError,
    },
    event_loop::EventLoop,
    frame_set::FrameSet,
    registry::RegistryType,
    renderer::{Renderer, RendererOptions},
    transformations::{image_renderer::Image, shader::Shader, web_renderer::WebRenderer},
    validation::SceneSpecExt,
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
                let shader_id = spec.shader_id.clone();

                match Shader::new(&ctx.wgpu_ctx, spec) {
                    Ok(shader) => Ok(guard
                        .renderers
                        .shaders
                        .register(shader_id, Arc::new(shader))?),
                    Err(err) => Err(RegisterRendererError::Shader(err, shader_id)),
                }
            }
            RendererSpec::WebRenderer(params) => {
                let instance_id = params.instance_id.clone();
                let web = Arc::new(WebRenderer::new(&ctx, params));

                Ok(guard.renderers.web_renderers.register(instance_id, web)?)
            }
            RendererSpec::Image(spec) => {
                let image_id = spec.image_id.clone();
                let asset = Image::new(&ctx, spec)
                    .map_err(|err| RegisterRendererError::Image(err, image_id.clone()))?;

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
        guard
            .scene_spec
            .validate_can_unregister(renderer_id, registry_type)?;
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

    pub fn update_scene(&mut self, scene_specs: Arc<SceneSpec>) -> Result<(), UpdateSceneError> {
        self.0.lock().unwrap().update_scene(scene_specs)
    }

    pub fn scene_spec(&self) -> Arc<SceneSpec> {
        self.0.lock().unwrap().scene_spec.clone()
    }
}
