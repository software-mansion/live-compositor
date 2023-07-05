use std::{collections::HashMap, error::Error};

use compositor_common::scene::TransformationRegistryKey;

use super::texture::Texture;

#[derive(Debug)]
pub enum TransformationParams {
    String(String),
    Binary(Vec<u8>),
}

pub trait Transformation: 'static {
    fn apply(
        &self,
        params: &TransformationParams,
        sources: &HashMap<String, Texture>,
        target: &Texture,
    ) -> Result<(), Box<dyn Error>>;

    fn registry_key(&self) -> TransformationRegistryKey;
}
