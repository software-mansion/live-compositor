use interleaved_yuv_to_rgba::InterleavedYuv422ToRgbaConverter;

use self::{planar_yuv_to_rgba::PlanarYuvToRgbaConverter, rgba_to_yuv::RgbaToYuvConverter};

use super::{
    texture::{InterleavedYuv422Texture, PlanarYuvTextures, RGBATexture},
    WgpuCtx,
};

mod interleaved_yuv_to_rgba;
mod planar_yuv_to_rgba;
mod rgba_to_yuv;

#[derive(Debug)]
pub struct TextureFormat {
    planar_yuv_to_rgba: PlanarYuvToRgbaConverter,
    interleaved_yuv_to_rgba: InterleavedYuv422ToRgbaConverter,
    rgba_to_yuv: RgbaToYuvConverter,
}

impl TextureFormat {
    pub fn new(device: &wgpu::Device) -> Self {
        let planar_yuv_to_rgba = PlanarYuvToRgbaConverter::new(device);
        let rgba_to_yuv = RgbaToYuvConverter::new(device);
        let interleaved_yuv_to_rgba = InterleavedYuv422ToRgbaConverter::new(device);
        Self {
            planar_yuv_to_rgba,
            rgba_to_yuv,
            interleaved_yuv_to_rgba,
        }
    }

    pub fn convert_rgba_to_yuv(
        &self,
        ctx: &WgpuCtx,
        src: (&RGBATexture, &wgpu::BindGroup),
        dst: &PlanarYuvTextures,
    ) {
        self.rgba_to_yuv.convert(ctx, src, dst);
    }

    pub fn convert_planar_yuv_to_rgba(
        &self,
        ctx: &WgpuCtx,
        src: (&PlanarYuvTextures, &wgpu::BindGroup),
        dst: &RGBATexture,
    ) {
        self.planar_yuv_to_rgba.convert(ctx, src, dst)
    }

    pub fn convert_interleaved_yuv_to_rgba(
        &self,
        ctx: &WgpuCtx,
        src: (&InterleavedYuv422Texture, &wgpu::BindGroup),
        dst: &RGBATexture,
    ) {
        self.interleaved_yuv_to_rgba.convert(ctx, src, dst)
    }
}
