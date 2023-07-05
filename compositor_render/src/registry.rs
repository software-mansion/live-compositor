use std::collections::HashMap;

use compositor_common::scene::TransformationRegistryKey;

use crate::renderer::transformation::Transformation;

#[derive(Debug, thiserror::Error)]
pub enum GetError {
    #[error("a transformation with a key {0} could not be found")]
    KeyNotFound(String),
}

#[derive(Debug, thiserror::Error)]
pub enum RegisterError {
    #[error("a transformation with a key {0} is already registered")]
    KeyTaken(String),
}

pub struct TransformationRegistry {
    registry: HashMap<TransformationRegistryKey, Box<dyn Transformation>>,
}

impl TransformationRegistry {
    pub fn new() -> Self {
        Self {
            registry: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn get(&self, key: &TransformationRegistryKey) -> Result<&dyn Transformation, GetError> {
        match self.registry.get(key) {
            Some(val) => Ok(&**val),
            None => Err(GetError::KeyNotFound(key.0.clone())),
        }
    }

    pub fn register(
        &mut self,
        transformation: Box<dyn Transformation>,
    ) -> Result<(), RegisterError> {
        let key = transformation.registry_key();

        if self.registry.contains_key(&key) {
            return Err(RegisterError::KeyTaken(key.0));
        }

        self.registry.insert(key, transformation);

        Ok(())
    }
}
