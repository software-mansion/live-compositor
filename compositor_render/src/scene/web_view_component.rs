use std::sync::Arc;

use crate::transformations::web_renderer::WebRenderer;

use super::{
    scene_state::BuildStateTreeCtx, BuildSceneError, Component, ComponentId, IntermediateNode,
    Size, StatefulComponent, WebViewComponent,
};

#[derive(Debug, Clone)]
pub(super) struct StatefulWebViewComponent {
    pub(super) id: Option<ComponentId>,
    pub(super) children: Vec<StatefulComponent>,
    pub(super) instance: Arc<WebRenderer>,
}

impl StatefulWebViewComponent {
    pub(super) fn component_id(&self) -> Option<&ComponentId> {
        self.id.as_ref()
    }

    pub(super) fn size(&self) -> Size {
        self.instance.resolution().into()
    }

    pub(super) fn intermediate_node(&self) -> Result<IntermediateNode, BuildSceneError> {
        let children = self
            .children
            .iter()
            .map(StatefulComponent::intermediate_node)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(IntermediateNode::WebView {
            web: self.clone(),
            children,
        })
    }
}

impl WebViewComponent {
    pub(super) fn stateful_component(
        mut self,
        ctx: &BuildStateTreeCtx,
    ) -> Result<StatefulComponent, BuildSceneError> {
        let instance = ctx
            .renderers
            .web_renderers
            .get(&self.instance_id)
            .ok_or_else(|| BuildSceneError::WebRendererNotFound(self.instance_id.clone()))?;

        let children = std::mem::take(&mut self.children)
            .into_iter()
            .map(|c| Component::stateful_component(c, ctx))
            .collect::<Result<_, _>>()?;
        Ok(StatefulComponent::WebView(StatefulWebViewComponent {
            id: self.id,
            instance,
            children,
        }))
    }
}
