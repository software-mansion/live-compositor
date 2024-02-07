use compositor_render::scene;

use super::*;

impl From<ComponentId> for scene::ComponentId {
    fn from(id: ComponentId) -> Self {
        Self(id.0)
    }
}

impl From<scene::ComponentId> for ComponentId {
    fn from(id: scene::ComponentId) -> Self {
        Self(id.0)
    }
}

impl From<RendererId> for compositor_render::RendererId {
    fn from(id: RendererId) -> Self {
        Self(id.0)
    }
}

impl From<compositor_render::RendererId> for RendererId {
    fn from(id: compositor_render::RendererId) -> Self {
        Self(id.0)
    }
}

impl From<OutputId> for compositor_render::OutputId {
    fn from(id: OutputId) -> Self {
        id.0.into()
    }
}

impl From<compositor_render::OutputId> for OutputId {
    fn from(id: compositor_render::OutputId) -> Self {
        Self(id.0)
    }
}

impl From<InputId> for compositor_render::InputId {
    fn from(id: InputId) -> Self {
        id.0.into()
    }
}

impl From<compositor_render::InputId> for InputId {
    fn from(id: compositor_render::InputId) -> Self {
        Self(id.0)
    }
}
