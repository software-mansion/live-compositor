use crate::{
    wgpu::{common_pipeline, WgpuCtx},
    Resolution,
};

use super::base::{copy_to_buffer, new_download_buffer, new_texture, TextureExt};

#[derive(Debug)]
pub struct RGBATexture {
    texture: wgpu::Texture,
    srgb_view: wgpu::TextureView,
    raw_view: wgpu::TextureView,
}

impl RGBATexture {
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
            wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            &[
                wgpu::TextureFormat::Rgba8UnormSrgb,
                wgpu::TextureFormat::Rgba8Unorm,
            ],
        );

        let srgb_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let raw_view = texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(wgpu::TextureFormat::Rgba8Unorm),
            ..Default::default()
        });

        Self {
            texture,
            srgb_view,
            raw_view,
        }
    }

    pub fn new_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        common_pipeline::create_single_texture_bgl(device)
    }

    // bing group for sRGB View (values in shader are mapped to linear)
    pub(super) fn new_bind_group_srgb(
        &self,
        ctx: &WgpuCtx,
        layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup {
        ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("texture bind group"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&self.srgb_view),
            }],
        })
    }

    // bing group for raw RGB View (values in shader are not converted)
    pub(super) fn new_bind_group_raw(
        &self,
        ctx: &WgpuCtx,
        layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup {
        ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("texture bind group"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&self.raw_view),
            }],
        })
    }

    pub fn upload(&self, ctx: &WgpuCtx, data: &[u8]) {
        self.texture.upload_data(&ctx.queue, data, 4);
    }

    pub fn new_download_buffer(&self, ctx: &WgpuCtx) -> wgpu::Buffer {
        new_download_buffer(&self.texture, ctx)
    }

    pub fn copy_to_buffer(&self, encoder: &mut wgpu::CommandEncoder, buffer: &wgpu::Buffer) {
        copy_to_buffer(&self.texture, encoder, buffer);
    }

    pub fn size(&self) -> wgpu::Extent3d {
        self.texture.size()
    }

    pub fn srgb_view(&self) -> &wgpu::TextureView {
        &self.srgb_view
    }

    pub fn raw_view(&self) -> &wgpu::TextureView {
        &self.raw_view
    }

    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    pub fn texture_owned(self) -> wgpu::Texture {
        self.texture
    }
}
