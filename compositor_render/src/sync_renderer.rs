use std::sync::{Arc, Mutex};

use compositor_common::{
    scene::{InputId, OutputId, SceneSpec},
    transformation::{TransformationRegistryKey, TransformationSpec},
};

use crate::{
    frame_set::FrameSet,
    renderer::{
        scene::SceneUpdateError, Renderer, RendererNewError, RendererRegisterTransformationError,
    },
    transformations::{shader::Shader, web_renderer::WebRenderer},
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
                let shader = Arc::new(Shader::new(&ctx, source));

                let mut guard = self.0.lock().unwrap();
                guard.shader_transforms.register(&key, shader)?
            }
            TransformationSpec::WebRenderer(params) => {
                let web = Arc::new(WebRenderer::new(&ctx, params)?);

                let mut guard = self.0.lock().unwrap();
                guard.web_renderers.register(&key, web)?
            }
        }
        Ok(())
    }

    pub fn render(&self, input: FrameSet<InputId>) -> FrameSet<OutputId> {
        self.0.lock().unwrap().render(input)
    }

    pub fn update_scene(&self, scene_specs: SceneSpec) -> Result<(), SceneUpdateError> {
        self.0.lock().unwrap().update_scene(scene_specs)
    }
}
