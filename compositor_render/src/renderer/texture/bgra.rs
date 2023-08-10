use std::ops::Deref;

use compositor_common::scene::Resolution;

use crate::renderer::WgpuCtx;

use super::RGBATexture;

pub struct BGRATexture(RGBATexture);

impl Deref for BGRATexture {
    type Target = RGBATexture;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl BGRATexture {
    pub fn new(ctx: &WgpuCtx, resolution: Resolution) -> Self {
        Self(RGBATexture::new(ctx, resolution))
    }
}
