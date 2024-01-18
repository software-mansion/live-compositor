use compositor_pipeline::pipeline;
use compositor_render::scene;

use crate::api::UpdateScene;

use super::util::*;
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

impl TryFrom<UpdateScene> for Vec<pipeline::OutputScene> {
    type Error = TypeError;

    fn try_from(update_scene: UpdateScene) -> Result<Self, Self::Error> {
        update_scene
            .outputs
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, TypeError>>()
    }
}

impl TryFrom<OutputScene> for pipeline::OutputScene {
    type Error = TypeError;

    fn try_from(scene: OutputScene) -> Result<Self, Self::Error> {
        Ok(pipeline::OutputScene {
            output_id: scene.output_id.into(),
            root: scene.root.try_into()?,
        })
    }
}
