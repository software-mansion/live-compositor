use std::{collections::HashMap, sync::Arc};

use compositor_common::renderer_spec::RendererId;

#[derive(Debug, thiserror::Error)]
pub enum RegisterError {
    #[error("Failed to register a {0}, The \"{1}\" {0} was already registered.")]
    KeyTaken(&'static str, Arc<str>),
}

pub enum RegistryType {
    Shader,
    WebRenderer,
    Image,
}

impl RegistryType {
    fn registry_item_name(&self) -> &'static str {
        match self {
            RegistryType::Shader => "shader",
            RegistryType::WebRenderer => "web renderer instance",
            RegistryType::Image => "image",
        }
    }
}

pub(crate) struct RendererRegistry<T: Clone> {
    registry: HashMap<RendererId, T>,
    registry_type: RegistryType,
}

impl<T: Clone> RendererRegistry<T> {
    pub(crate) fn new(registry_type: RegistryType) -> Self {
        Self {
            registry: HashMap::new(),
            registry_type,
        }
    }

    pub(crate) fn get(&self, key: &RendererId) -> Option<T> {
        self.registry.get(key).map(Clone::clone)
    }

    pub(crate) fn register(&mut self, id: RendererId, renderer: T) -> Result<(), RegisterError> {
        if self.registry.contains_key(&id) {
            return Err(RegisterError::KeyTaken(
                self.registry_type.registry_item_name(),
                id.0.clone(),
            ));
        }

        self.registry.insert(id.clone(), renderer);

        Ok(())
    }
}
