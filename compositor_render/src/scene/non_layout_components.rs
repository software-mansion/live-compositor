use super::{BaseNode, BuildSceneError, Component, InputStreamComponent, ShaderComponent};

impl ShaderComponent {
    pub(super) fn base_node(&self) -> Result<BaseNode, BuildSceneError> {
        let children = self
            .children
            .iter()
            .map(Component::base_node)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(BaseNode::Shader {
            shader: self.clone(),
            children,
        })
    }
}

impl InputStreamComponent {
    pub(super) fn base_node(&self) -> Result<BaseNode, BuildSceneError> {
        Ok(BaseNode::InputStream(self.clone()))
    }
}
