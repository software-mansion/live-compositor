use std::marker::PhantomData;

use bytes::Bytes;
use compositor_common::{frame::YuvData, scene::Resolution};
use wgpu::Buffer;

use crate::renderer::WgpuCtx;

use super::{base::Texture, utils::pad_to_256};

pub struct YUVPendingDownload<'a, F, E>
where
    F: FnOnce() -> Result<Bytes, E> + 'a,
{
    y: F,
    u: F,
    v: F,
    _phantom: PhantomData<&'a F>,
}

impl<'a, F, E> YUVPendingDownload<'a, F, E>
where
    F: FnOnce() -> Result<Bytes, E>,
{
    pub(super) fn new(y: F, u: F, v: F) -> Self {
        Self {
            y,
            u,
            v,
            _phantom: Default::default(),
        }
    }

    /// `device.poll(wgpu::MaintainBase::Wait)` needs to be called after download
    /// is started, but before this method is called.
    pub fn wait(self) -> Result<YuvData, E> {
        let YUVPendingDownload { y, u, v, _phantom } = self;
        Ok(YuvData {
            y_plane: y()?,
            u_plane: u()?,
            v_plane: v()?,
        })
    }
}

pub struct YUVTextures {
    pub(super) plains: [Texture; 3],
}

impl YUVTextures {
    pub fn new(ctx: &WgpuCtx, resolution: &Resolution) -> Self {
        Self {
            plains: [
                Self::new_plane(ctx, resolution.width, resolution.height),
                Self::new_plane(ctx, resolution.width / 2, resolution.height / 2),
                Self::new_plane(ctx, resolution.width / 2, resolution.height / 2),
            ],
        }
    }

    pub fn plain(&self, i: usize) -> &Texture {
        &self.plains[i]
    }

    fn new_plane(ctx: &WgpuCtx, width: usize, height: usize) -> Texture {
        Texture::new(
            ctx,
            None,
            wgpu::Extent3d {
                width: width as u32,
                height: height as u32,
                depth_or_array_layers: 1,
            },
            wgpu::TextureFormat::R8Unorm,
            wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
        )
    }

    pub fn new_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        let create_entry = |binding: u32| wgpu::BindGroupLayoutEntry {
            binding,
            ty: Texture::DEFAULT_BINDING_TYPE,
            visibility: wgpu::ShaderStages::FRAGMENT,
            count: None,
        };
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("yuv all textures bind group layout"),
            entries: &[create_entry(0), create_entry(1), create_entry(2)],
        })
    }

    pub(super) fn new_bind_group(
        &self,
        ctx: &WgpuCtx,
        layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup {
        ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("yuv all textures bind group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.plains[0].view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&self.plains[1].view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&self.plains[2].view),
                },
            ],
        })
    }

    pub(super) fn new_download_buffers(&self, ctx: &WgpuCtx) -> [Buffer; 3] {
        [
            self.new_download_buffer(ctx, 0),
            self.new_download_buffer(ctx, 1),
            self.new_download_buffer(ctx, 2),
        ]
    }

    fn new_download_buffer(&self, ctx: &WgpuCtx, plain: usize) -> Buffer {
        let size = self.plains[plain].size();
        ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("output texture buffer"),
            mapped_at_creation: false,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            size: (pad_to_256(size.width) * size.height) as u64,
        })
    }

    pub(super) fn copy_to_buffers(&self, ctx: &WgpuCtx, buffers: &[Buffer; 3]) {
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("transfer result yuv texture to buffers encoder"),
            });

        for plane in [0, 1, 2] {
            let texture = &self.plains[plane].texture;
            let size = texture.size();
            encoder.copy_texture_to_buffer(
                wgpu::ImageCopyTexture {
                    aspect: wgpu::TextureAspect::All,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    texture,
                },
                wgpu::ImageCopyBuffer {
                    buffer: &buffers[plane],
                    layout: wgpu::ImageDataLayout {
                        bytes_per_row: Some(pad_to_256(size.width)),
                        rows_per_image: Some(size.height),
                        offset: 0,
                    },
                },
                size,
            )
        }

        ctx.queue.submit(Some(encoder.finish()));
    }

    pub fn upload(&self, ctx: &WgpuCtx, data: &YuvData) {
        self.plains[0].upload_data(&ctx.queue, &data.y_plane, 1);
        self.plains[1].upload_data(&ctx.queue, &data.u_plane, 1);
        self.plains[2].upload_data(&ctx.queue, &data.v_plane, 1);
    }
}
