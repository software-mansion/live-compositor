use std::{collections::HashMap, error::Error, rc::Rc};

use compositor_common::scene::TransformationRegistryKey;

use super::{texture::Texture, WgpuCtx};

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

    fn new(ctx: Rc<WgpuCtx>) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized;
}
