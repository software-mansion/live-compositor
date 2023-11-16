use super::{
    scene_state::BuildStateTreeCtx, BuildSceneError, ComponentId, InputStreamComponent,
    IntermediateNode, Size, StatefulComponent,
};

#[derive(Debug, Clone)]
pub(super) struct StatefulInputStreamComponent {
    pub(super) component: InputStreamComponent,
    pub(super) size: Option<Size>,
}

impl StatefulInputStreamComponent {
    pub(super) fn component_id(&self) -> Option<&ComponentId> {
        self.component.id.as_ref()
    }

    pub(super) fn intermediate_node(&self) -> Result<IntermediateNode, BuildSceneError> {
        Ok(IntermediateNode::InputStream(self.clone()))
    }
}

impl InputStreamComponent {
    pub(super) fn stateful_component(self, _ctx: &BuildStateTreeCtx) -> StatefulComponent {
        StatefulComponent::InputStream(StatefulInputStreamComponent {
            component: self,
            size: None, // TODO: get from ctx
        })
    }
}
