use super::{
    scene_state::BuildStateTreeCtx, BuildSceneError, ComponentId, ImageComponent, IntermediateNode,
    Size, StatefulComponent,
};

#[derive(Debug, Clone)]
pub(super) struct StatefulImageComponent {
    pub(super) component: ImageComponent,
    pub(super) size: Option<Size>,
}

impl StatefulImageComponent {
    pub(super) fn component_id(&self) -> Option<&ComponentId> {
        self.component.id.as_ref()
    }

    pub(super) fn intermediate_node(&self) -> Result<IntermediateNode, BuildSceneError> {
        Ok(IntermediateNode::Image(self.clone()))
    }
}

impl ImageComponent {
    pub(super) fn stateful_component(self, _ctx: &BuildStateTreeCtx) -> StatefulComponent {
        StatefulComponent::Image(StatefulImageComponent {
            component: self,
            size: None, // TODO: get from ctx
        })
    }
}
