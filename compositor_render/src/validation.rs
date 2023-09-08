use compositor_common::{
    renderer_spec::RendererId,
    scene::{NodeParams, SceneSpec},
};

use crate::{error::UnregisterRendererError, registry::RegistryType};

pub trait SceneSpecExt {
    fn validate_can_unregister(
        &self,
        renderer_id: &RendererId,
        registry_type: RegistryType,
    ) -> Result<(), UnregisterRendererError>;
}

impl SceneSpecExt for SceneSpec {
    fn validate_can_unregister(
        &self,
        renderer_id: &RendererId,
        registry_type: RegistryType,
    ) -> Result<(), UnregisterRendererError> {
        match registry_type {
            RegistryType::Shader => {
                let node = self.nodes.iter().find(|node| match &node.params {
                    NodeParams::Shader { shader_id, .. } => shader_id == renderer_id,
                    _ => false,
                });
                if let Some(node) = node {
                    return Err(UnregisterRendererError::ShaderStillInUse(
                        renderer_id.clone(),
                        node.node_id.clone(),
                    ));
                }
            }
            RegistryType::WebRenderer => {
                let node = self.nodes.iter().find(|node| match &node.params {
                    NodeParams::WebRenderer { instance_id, .. } => instance_id == renderer_id,
                    _ => false,
                });
                if let Some(node) = node {
                    return Err(UnregisterRendererError::WebRendererInstanceStillInUse(
                        renderer_id.clone(),
                        node.node_id.clone(),
                    ));
                }
            }
            RegistryType::Image => {
                let node = self.nodes.iter().find(|node| match &node.params {
                    NodeParams::Image { image_id, .. } => image_id == renderer_id,
                    _ => false,
                });
                if let Some(node) = node {
                    return Err(UnregisterRendererError::ImageStillInUse(
                        renderer_id.clone(),
                        node.node_id.clone(),
                    ));
                }
            }
        }
        Ok(())
    }
}
