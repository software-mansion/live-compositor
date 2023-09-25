use crate::renderer::texture::base::Texture;
use crate::renderer::texture::rgba::RGBATexture;
use crate::renderer::texture::{NodeTexture, NodeTextureState};
use compositor_common::scene::{NodeId, Resolution};

pub(crate) fn pad_to_256(value: u32) -> u32 {
    value + (256 - (value % 256))
}

pub fn texture_size_to_resolution(size: &wgpu::Extent3d) -> Resolution {
    Resolution {
        width: size.width as usize,
        height: size.height as usize,
    }
}

pub fn sources_to_textures<'a>(sources: &'a [(&NodeId, &NodeTexture)]) -> Vec<Option<&'a Texture>> {
    sources
        .iter()
        .map(|(_, texture)| {
            texture
                .state()
                .map(NodeTextureState::rgba_texture)
                .map(RGBATexture::texture)
        })
        .collect()
}
