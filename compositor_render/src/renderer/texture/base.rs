use crate::renderer::WgpuCtx;

use super::utils::pad_to_256;

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
}

impl Texture {
    pub(super) const DEFAULT_BINDING_TYPE: wgpu::BindingType = wgpu::BindingType::Texture {
        sample_type: wgpu::TextureSampleType::Float { filterable: true },
        view_dimension: wgpu::TextureViewDimension::D2,
        multisampled: false,
    };
    pub fn new(
        ctx: &WgpuCtx,
        label: Option<&str>,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
    ) -> Self {
        let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[format],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self { texture, view }
    }

    pub fn size(&self) -> wgpu::Extent3d {
        self.texture.size()
    }

    pub(super) fn upload_data(&self, queue: &wgpu::Queue, data: &[u8], bytes_per_pixel: u32) {
        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                texture: &self.texture,
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(self.texture.width() * bytes_per_pixel),
                rows_per_image: Some(self.texture.height()),
            },
            self.texture.size(),
        );
    }

    /// Returns `None` for some depth formats
    pub(super) fn block_size(&self) -> Option<u32> {
        self.texture.format().block_size(None)
    }

    pub(super) fn download_buffer(&self, ctx: &WgpuCtx) -> wgpu::Buffer {
        let size = self.size();
        let block_size = self.block_size().unwrap();

        ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("texture buffer"),
            mapped_at_creation: false,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            size: (block_size * pad_to_256(size.width) * size.height) as u64,
        })
    }

    /// [`wgpu::Queue::submit`] has to be called afterwards
    pub(super) fn copy_to_buffer(&self, encoder: &mut wgpu::CommandEncoder, buffer: &wgpu::Buffer) {
        let size = self.size();
        let block_size = self.block_size().unwrap();

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                texture: &self.texture,
            },
            wgpu::ImageCopyBuffer {
                buffer,
                layout: wgpu::ImageDataLayout {
                    bytes_per_row: Some(block_size * pad_to_256(size.width)),
                    rows_per_image: Some(size.height),
                    offset: 0,
                },
            },
            size,
        );
    }
}
