use compositor_render::scene;

use super::*;

impl From<ComponentId> for scene::ComponentId {
    fn from(id: ComponentId) -> Self {
        Self(id.0)
    }
}

impl From<RendererId> for compositor_render::RendererId {
    fn from(id: RendererId) -> Self {
        Self(id.0)
    }
}

impl From<OutputId> for compositor_render::OutputId {
    fn from(id: OutputId) -> Self {
        id.0.into()
    }
}

impl From<InputId> for compositor_render::InputId {
    fn from(id: InputId) -> Self {
        id.0.into()
    }
}
