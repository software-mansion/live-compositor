use std::{collections::HashMap, sync::Arc};

use compositor_common::renderer_spec::RendererId;

#[derive(Debug, thiserror::Error)]
pub enum GetError {
    #[error("a {0} with a key {1} could not be found")]
    KeyNotFound(&'static str, Arc<str>),
}

#[derive(Debug, thiserror::Error)]
pub enum RegisterError {
    #[error("a {0} with a key {1} is already registered")]
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
            RegistryType::Shader => "shader transformation",
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

    #[allow(dead_code)]
    pub(crate) fn get(&self, key: &RendererId) -> Result<T, GetError> {
        match self.registry.get(key) {
            Some(val) => Ok(val.clone()),
            None => Err(GetError::KeyNotFound(
                self.registry_type.registry_item_name(),
                key.0.clone(),
            )),
        }
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
