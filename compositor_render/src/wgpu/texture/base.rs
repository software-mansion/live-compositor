use crate::wgpu::WgpuCtx;

use super::utils::pad_to_256;

pub(crate) trait TextureExt {
    fn copy_wgpu_texture(&self, ctx: &WgpuCtx) -> wgpu::Texture;
    fn fill_from_wgpu_texture(
        &self,
        ctx: &WgpuCtx,
        source: &wgpu::Texture,
    ) -> Result<(), TextureCopyError>;
    fn empty(device: &wgpu::Device) -> Self;
    fn upload_data(&self, queue: &wgpu::Queue, data: &[u8], bytes_per_pixel: u32);
    fn block_size(&self) -> Option<u32>;
}

pub(super) const DEFAULT_BINDING_TYPE: wgpu::BindingType = wgpu::BindingType::Texture {
    sample_type: wgpu::TextureSampleType::Float { filterable: true },
    view_dimension: wgpu::TextureViewDimension::D2,
    multisampled: false,
};

impl TextureExt for wgpu::Texture {
    fn copy_wgpu_texture(&self, ctx: &WgpuCtx) -> wgpu::Texture {
        let destination = ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: self.size(),
            mip_level_count: self.mip_level_count(),
            sample_count: self.sample_count(),
            dimension: self.dimension(),
            format: self.format(),
            usage: self.usage(),
            view_formats: &[self.format()],
        });
        copy_texture_to_texture(ctx, &self, &destination);
        destination
    }

    fn fill_from_wgpu_texture(
        &self,
        ctx: &WgpuCtx,
        source: &wgpu::Texture,
    ) -> Result<(), TextureCopyError> {
        let expected = (
            self.size(),
            self.mip_level_count(),
            self.sample_count(),
            self.dimension(),
            self.format(),
            self.usage(),
        );
        let actual = (
            source.size(),
            source.mip_level_count(),
            source.sample_count(),
            source.dimension(),
            source.format(),
            source.usage(),
        );

        if expected != actual {
            return Err(TextureCopyError {
                expected: format!("{expected:?}"),
                actual: format!("{actual:?}"),
            });
        }
        copy_texture_to_texture(ctx, source, &self);
        Ok(())
    }

    fn empty(device: &wgpu::Device) -> wgpu::Texture {
        new_texture(
            device,
            Some("empty texture"),
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureUsages::TEXTURE_BINDING,
            &[
                wgpu::TextureFormat::Rgba8UnormSrgb,
                wgpu::TextureFormat::Rgba8Unorm,
            ],
        )
    }

    fn upload_data(&self, queue: &wgpu::Queue, data: &[u8], bytes_per_pixel: u32) {
        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                texture: &self,
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(self.width() * bytes_per_pixel),
                rows_per_image: Some(self.height()),
            },
            self.size(),
        );
    }

    /// Returns `None` for some depth formats
    fn block_size(&self) -> Option<u32> {
        self.format().block_copy_size(None)
    }
}

pub fn new_texture(
    device: &wgpu::Device,
    label: Option<&str>,
    size: wgpu::Extent3d,
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
    view_formats: &[wgpu::TextureFormat],
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label,
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage,
        view_formats,
    })
}

pub(super) fn new_download_buffer(texture: &wgpu::Texture, ctx: &WgpuCtx) -> wgpu::Buffer {
    let size = texture.size();
    let block_size = texture.block_size().unwrap();

    ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("texture buffer"),
        mapped_at_creation: false,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        size: (pad_to_256(block_size * size.width) * size.height) as u64,
    })
}

/// [`wgpu::Queue::submit`] has to be called afterwards
pub(super) fn copy_to_buffer(
    texture: &wgpu::Texture,
    encoder: &mut wgpu::CommandEncoder,
    buffer: &wgpu::Buffer,
) {
    let size = texture.size();
    let block_size = texture.block_size().unwrap();

    encoder.copy_texture_to_buffer(
        wgpu::ImageCopyTexture {
            aspect: wgpu::TextureAspect::All,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            texture: &texture,
        },
        wgpu::ImageCopyBuffer {
            buffer,
            layout: wgpu::ImageDataLayout {
                bytes_per_row: Some(pad_to_256(block_size * size.width)),
                rows_per_image: Some(size.height),
                offset: 0,
            },
        },
        size,
    );
}

#[derive(Debug, thiserror::Error)]
#[error("Passed invalid texture. Expected: {expected}, Actual: {actual}")]
pub struct TextureCopyError {
    expected: String,
    actual: String,
}

fn copy_texture_to_texture(ctx: &WgpuCtx, source: &wgpu::Texture, destination: &wgpu::Texture) {
    let mut encoder = ctx.device.create_command_encoder(&Default::default());

    encoder.copy_texture_to_texture(
        source.as_image_copy(),
        destination.as_image_copy(),
        source.size(),
    );
    ctx.queue.submit(Some(encoder.finish()));
}
