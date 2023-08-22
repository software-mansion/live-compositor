use std::sync::{Arc, Mutex};

use compositor_common::{
    scene::{InputId, OutputId, SceneSpec},
    transformation::{TransformationRegistryKey, TransformationSpec},
    Framerate,
};

use crate::{
    event_loop::EventLoop,
    frame_set::FrameSet,
    renderer::{
        scene::SceneUpdateError, RenderError, Renderer, RendererNewError,
        RendererRegisterTransformationError,
    },
    transformations::{image_renderer::Image, shader::Shader, web_renderer::WebRenderer},
    WebRendererOptions,
};

#[derive(Clone)]
pub struct SyncRenderer(Arc<Mutex<Renderer>>);

impl SyncRenderer {
    pub fn new(
        web_renderer_opts: WebRendererOptions,
        web_renderer_framerate: Framerate,
    ) -> Result<(Self, EventLoop), RendererNewError> {
        let renderer = Renderer::new(web_renderer_opts, web_renderer_framerate)?;
        let event_loop = EventLoop::new(renderer.chromium_context.cef_context());

        Ok((Self(Arc::new(Mutex::new(renderer))), event_loop))
    }

    pub fn register_transformation(
        &self,
        key: TransformationRegistryKey,
        spec: TransformationSpec,
    ) -> Result<(), RendererRegisterTransformationError> {
        let ctx = self.0.lock().unwrap().register_transformation_ctx();
        match spec {
            TransformationSpec::Shader { source } => {
                let shader = Arc::new(Shader::new(&ctx, source)?);

                let mut guard = self.0.lock().unwrap();
                guard.shader_transforms.register(&key, shader)?
            }
            TransformationSpec::WebRenderer(params) => {
                let web = Arc::new(WebRenderer::new(&ctx, params)?);

                let mut guard = self.0.lock().unwrap();
                guard.web_renderers.register(&key, web)?
            }
            TransformationSpec::Image(spec) => {
                let asset = Image::new(&ctx, spec)?;

                let mut guard = self.0.lock().unwrap();
                guard.image_registry.register(&key, asset)?
            }
        }
        Ok(())
    }

    pub fn render(&self, input: FrameSet<InputId>) -> Result<FrameSet<OutputId>, RenderError> {
        self.0.lock().unwrap().render(input)
    }

    pub fn update_scene(&mut self, scene_specs: Arc<SceneSpec>) -> Result<(), SceneUpdateError> {
        self.0.lock().unwrap().update_scene(scene_specs)
    }

    pub fn scene_spec(&self) -> Arc<SceneSpec> {
        self.0.lock().unwrap().scene_spec.clone()
    }
}
