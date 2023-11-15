use compositor_common::{renderer_spec::RendererId, scene::shader::ShaderParam};

use super::{
    scene_state::BuildStateTreeCtx, BuildSceneError, Component, ComponentId, ComponentState,
    IntermediateNode, ShaderComponent, Size,
};

#[derive(Debug, Clone)]
pub(super) struct ShaderComponentState {
    pub(super) component: ShaderComponentParams,
    pub(super) children: Vec<ComponentState>,
}

#[derive(Debug, Clone)]
pub(crate) struct ShaderComponentParams {
    pub(crate) id: Option<ComponentId>,
    pub(crate) shader_id: RendererId,
    pub(crate) shader_param: Option<ShaderParam>,
    pub(crate) size: Size,
}

impl ShaderComponentState {
    pub(super) fn component_id(&self) -> Option<&ComponentId> {
        self.component.id.as_ref()
    }

    pub(super) fn base_node(&self) -> Result<IntermediateNode, BuildSceneError> {
        let children = self
            .children
            .iter()
            .map(ComponentState::base_node)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(IntermediateNode::Shader {
            shader: self.clone(),
            children,
        })
    }
}

impl ShaderComponent {
    pub(super) fn state_component(mut self, ctx: &BuildStateTreeCtx) -> ComponentState {
        let children = std::mem::take(&mut self.children)
            .into_iter()
            .map(|c| Component::state_component(c, ctx))
            .collect();
        ComponentState::Shader(ShaderComponentState {
            component: ShaderComponentParams {
                id: self.id,
                shader_id: self.shader_id,
                shader_param: self.shader_param,
                size: self.size,
            },
            children,
        })
    }
}
