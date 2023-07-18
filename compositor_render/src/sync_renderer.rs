use std::sync::{Arc, Mutex};

use compositor_common::{
    scene::{InputId, OutputId, SceneSpec},
    transformation::{TransformationRegistryKey, TransformationSpec},
};

use crate::{
    frameset::FrameSet,
    renderer::{
        scene::SceneUpdateError, Renderer, RendererNewError, RendererRegisterTransformationError,
        RendererRenderError,
    },
};

#[derive(Clone)]
pub struct SyncRenderer(Arc<Mutex<Renderer>>);

impl SyncRenderer {
    pub fn new() -> Result<Self, RendererNewError> {
        Ok(Self(Arc::new(Mutex::new(Renderer::new()?))))
    }
    pub fn register_transformation(
        &self,
        key: TransformationRegistryKey,
        spec: TransformationSpec,
    ) -> Result<(), RendererRegisterTransformationError> {
        self.0.lock().unwrap().register_transformation(key, spec)
    }

    pub fn render(
        &self,
        input: FrameSet<InputId>,
    ) -> Result<FrameSet<OutputId>, RendererRenderError> {
        self.0.lock().unwrap().render(input)
    }

    pub fn update_scene(&self, scene_specs: SceneSpec) -> Result<(), SceneUpdateError> {
        self.0.lock().unwrap().update_scene(scene_specs)
    }
}
