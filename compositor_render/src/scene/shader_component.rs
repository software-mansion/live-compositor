use std::sync::Arc;

use compositor_common::{renderer_spec::RendererId, scene::shader::ShaderParam};

use crate::transformations::shader::Shader;

use super::{
    scene_state::BuildStateTreeCtx, BuildSceneError, Component, ComponentId, IntermediateNode,
    ShaderComponent, Size, StatefulComponent,
};

#[derive(Debug, Clone)]
pub(super) struct StatefulShaderComponent {
    pub(super) component: ShaderComponentParams,
    pub(super) children: Vec<StatefulComponent>,
    pub(super) shader: Arc<Shader>,
}

#[derive(Debug, Clone)]
pub(crate) struct ShaderComponentParams {
    pub(crate) id: Option<ComponentId>,
    pub(crate) shader_id: RendererId,
    pub(crate) shader_param: Option<ShaderParam>,
    pub(crate) size: Size,
}

impl StatefulShaderComponent {
    pub(super) fn component_id(&self) -> Option<&ComponentId> {
        self.component.id.as_ref()
    }

    pub(super) fn intermediate_node(&self) -> Result<IntermediateNode, BuildSceneError> {
        let children = self
            .children
            .iter()
            .map(StatefulComponent::intermediate_node)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(IntermediateNode::Shader {
            shader: self.clone(),
            children,
        })
    }
}

impl ShaderComponent {
    pub(super) fn stateful_component(
        self,
        ctx: &BuildStateTreeCtx,
    ) -> Result<StatefulComponent, BuildSceneError> {
        let shader = ctx
            .renderers
            .shaders
            .get(&self.shader_id)
            .ok_or_else(|| BuildSceneError::ShaderNotFound(self.shader_id.clone()))?;

        let children = self
            .children
            .into_iter()
            .map(|c| Component::stateful_component(c, ctx))
            .collect::<Result<_, _>>()?;
        Ok(StatefulComponent::Shader(StatefulShaderComponent {
            component: ShaderComponentParams {
                id: self.id,
                shader_id: self.shader_id,
                shader_param: self.shader_param,
                size: self.size,
            },
            shader,
            children,
        }))
    }
}
