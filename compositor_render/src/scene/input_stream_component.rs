use compositor_common::scene::Resolution;

use super::{
    scene_state::BuildStateTreeCtx, BaseNode, BuildSceneError, ComponentId, ComponentState,
    InputStreamComponent,
};

#[derive(Debug, Clone)]
pub(super) struct InputStreamComponentState {
    pub(super) component: InputStreamComponent,
    pub(super) size: Option<Resolution>,
}

impl InputStreamComponentState {
    pub(super) fn component_id(&self) -> Option<&ComponentId> {
        self.component.id.as_ref()
    }

    pub(super) fn base_node(&self) -> Result<BaseNode, BuildSceneError> {
        Ok(BaseNode::InputStream(self.clone()))
    }
}

impl InputStreamComponent {
    pub(super) fn state_component(self, _ctx: &BuildStateTreeCtx) -> ComponentState {
        ComponentState::InputStream(InputStreamComponentState {
            component: self,
            size: None, // TODO: get from ctx
        })
    }
}
