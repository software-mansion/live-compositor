use std::{collections::HashMap, sync::Arc};

use compositor_common::transformation::TransformationRegistryKey;

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
}

impl RegistryType {
    fn registry_item_name(&self) -> &'static str {
        match self {
            RegistryType::Shader => "shader transformation",
            RegistryType::WebRenderer => "web renderer instance",
        }
    }
}

pub(crate) struct TransformationRegistry<T: Clone> {
    registry: HashMap<TransformationRegistryKey, T>,
    registry_type: RegistryType,
}

impl<T: Clone> TransformationRegistry<T> {
    pub(crate) fn new(registry_type: RegistryType) -> Self {
        Self {
            registry: HashMap::new(),
            registry_type,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn get(&self, key: &TransformationRegistryKey) -> Result<T, GetError> {
        match self.registry.get(key) {
            Some(val) => Ok(val.clone()),
            None => Err(GetError::KeyNotFound(
                self.registry_type.registry_item_name(),
                key.0.clone(),
            )),
        }
    }

    pub(crate) fn register(
        &mut self,
        key: &TransformationRegistryKey,
        transformation: T,
    ) -> Result<(), RegisterError> {
        if self.registry.contains_key(key) {
            return Err(RegisterError::KeyTaken(
                self.registry_type.registry_item_name(),
                key.0.clone(),
            ));
        }

        self.registry.insert(key.clone(), transformation);

        Ok(())
    }
}
