use self::{rgba_to_yuv::RGBAToYUVConverter, yuv_to_rgba::YUVToRGBAConverter};

use super::{
    texture::{RGBATexture, YUVTextures},
    WgpuCtx,
};

mod rgba_to_yuv;
mod yuv_to_rgba;

pub struct TextureFormat {
    yuv_to_rgba: YUVToRGBAConverter,
    rgba_to_yuv: RGBAToYUVConverter,

    yuv_layout: wgpu::BindGroupLayout,
    rgba_layout: wgpu::BindGroupLayout,
}

impl TextureFormat {
    pub fn new(device: &wgpu::Device) -> Self {
        let yuv_layout = YUVTextures::new_bind_group_layout(device);
        let rgba_layout = RGBATexture::new_bind_group_layout(device);
        let yuv_to_rgba = YUVToRGBAConverter::new(device, &yuv_layout);
        let rgba_to_yuv = RGBAToYUVConverter::new(device, &rgba_layout);
        Self {
            yuv_to_rgba,
            rgba_to_yuv,

            yuv_layout,
            rgba_layout,
        }
    }

    pub fn yuv_layout(&self) -> &wgpu::BindGroupLayout {
        &self.yuv_layout
    }

    pub fn rgba_layout(&self) -> &wgpu::BindGroupLayout {
        &self.rgba_layout
    }

    pub fn convert_rgba_to_yuv(
        &self,
        ctx: &WgpuCtx,
        src: (&RGBATexture, &wgpu::BindGroup),
        dst: &YUVTextures,
    ) {
        self.rgba_to_yuv.convert(ctx, src, dst);
    }

    pub fn convert_yuv_to_rgba(
        &self,
        ctx: &WgpuCtx,
        src: (&YUVTextures, &wgpu::BindGroup),
        dst: &RGBATexture,
    ) {
        self.yuv_to_rgba.convert(ctx, src, dst)
    }
}
