use compositor_common::scene::Resolution;

use super::{
    scene_state::BuildStateTreeCtx, BuildSceneError, ComponentId, InputStreamComponent,
    IntermediateNode, Size, StatefulComponent,
};

#[derive(Debug, Clone)]
pub(super) struct StatefulInputStreamComponent {
    pub(super) component: InputStreamComponent,
    pub(super) size: Size,
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
    pub(super) fn stateful_component(
        self,
        ctx: &BuildStateTreeCtx,
    ) -> Result<StatefulComponent, BuildSceneError> {
        let input = ctx
            .input_resolutions
            .get(&self.input_id)
            .copied()
            .unwrap_or(Resolution {
                width: 0,
                height: 0,
            });
        Ok(StatefulComponent::InputStream(
            StatefulInputStreamComponent {
                component: self,
                size: input.into(),
            },
        ))
    }
}
