use std::{error::Error, rc::Rc};

use compositor_common::scene::TransformationRegistryKey;

use super::{texture::Texture, WgpuCtx};

pub trait Transformation: 'static {
    fn apply(
        &self,
        params: &[u8],
        source: &Texture,
        target: &Texture,
    ) -> Result<(), Box<dyn Error>>;

    fn registry_key(&self) -> TransformationRegistryKey;

    fn new(ctx: Rc<WgpuCtx>) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized;
}
