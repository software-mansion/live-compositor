use crate::{
    wgpu::{
        texture::{RGBATexture, Texture},
        WgpuCtx,
    },
    Resolution,
};

#[derive(Debug)]
pub struct BGRATexture(Texture);

impl BGRATexture {
    pub fn new(ctx: &WgpuCtx, resolution: Resolution) -> Self {
        Self(Texture::new(
            &ctx.device,
            None,
            wgpu::Extent3d {
                width: resolution.width as u32,
                height: resolution.height as u32,
                depth_or_array_layers: 1,
            },
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        ))
    }

    pub fn upload(&self, ctx: &WgpuCtx, data: &[u8]) {
        self.0.upload_data(&ctx.queue, data, 4);
    }

    pub fn texture(&self) -> &Texture {
        &self.0
    }
}

pub(super) trait RGBATextureExt {
    fn copy_to_buffer(&self, encoder: &mut wgpu::CommandEncoder, buffer: &wgpu::Buffer);
}

impl RGBATextureExt for RGBATexture {
    fn copy_to_buffer(&self, encoder: &mut wgpu::CommandEncoder, buffer: &wgpu::Buffer) {
        self.texture().copy_to_buffer(encoder, buffer);
    }
}
