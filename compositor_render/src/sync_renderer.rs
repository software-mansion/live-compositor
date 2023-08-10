use std::sync::{Arc, Mutex};

use compositor_common::{
    scene::{InputId, OutputId, SceneSpec},
    transformation::{TransformationRegistryKey, TransformationSpec},
};

use crate::{
    frame_set::FrameSet,
    renderer::{
        scene::SceneUpdateError, RenderError, Renderer, RendererNewError,
        RendererRegisterTransformationError,
    },
    transformations::{image_renderer::Image, shader::Shader, web_renderer::WebRenderer},
};

#[derive(Clone)]
pub struct SyncRenderer(Arc<Mutex<Renderer>>);

impl SyncRenderer {
    pub fn new(init_web: bool) -> Result<Self, RendererNewError> {
        Ok(Self(Arc::new(Mutex::new(Renderer::new(init_web)?))))
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
                let asset = Arc::new(Image::new(&ctx, spec)?);

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
