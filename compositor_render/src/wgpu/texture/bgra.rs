use crate::{wgpu::WgpuCtx, Resolution};

use super::{base::new_texture, TextureExt};

#[derive(Debug)]
pub struct BGRATexture {
    texture: wgpu::Texture,
    srgb_view: wgpu::TextureView,
}

impl BGRATexture {
    pub fn new(ctx: &WgpuCtx, resolution: Resolution) -> Self {
        let texture = new_texture(
            &ctx.device,
            None,
            wgpu::Extent3d {
                width: resolution.width as u32,
                height: resolution.height as u32,
                depth_or_array_layers: 1,
            },
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            &[
                wgpu::TextureFormat::Rgba8UnormSrgb,
                wgpu::TextureFormat::Rgba8Unorm,
            ],
        );
        let view = texture.create_view(&Default::default());
        Self {
            texture,
            srgb_view: view,
        }
    }

    pub fn upload(&self, ctx: &WgpuCtx, data: &[u8]) {
        self.texture.upload_data(&ctx.queue, data, 4);
    }

    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    pub fn srgb_view(&self) -> &wgpu::TextureView {
        &self.srgb_view
    }
}
