use std::{io::Write, sync::Arc};

use bytes::{BufMut, Bytes, BytesMut};
use compositor_common::{frame::YuvData, scene::Resolution, Frame};
use wgpu::MapMode;

use super::WgpuCtx;

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
}

impl Texture {
    pub const DEFAULT_BINDING_TYPE: wgpu::BindingType = wgpu::BindingType::Texture {
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

    pub fn upload_frame_to_textures(ctx: &WgpuCtx, textures: &[Texture; 3], frame: Arc<Frame>) {
        // TODO maybe resize?
        textures[0].upload_data(&ctx.queue, &frame.data.y_plane, 1);
        textures[1].upload_data(&ctx.queue, &frame.data.u_plane, 1);
        textures[2].upload_data(&ctx.queue, &frame.data.v_plane, 1);
        // TODO: https://github.com/membraneframework/video_compositor/pull/30#discussion_r1277993507
        ctx.queue.submit([]);
    }

    fn upload_data(&self, queue: &wgpu::Queue, data: &[u8], bytes_per_pixel: u32) {
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
}

pub struct YUVTextures(pub [Texture; 3]);

impl YUVTextures {
    pub fn new(ctx: &WgpuCtx, resolution: &Resolution) -> Self {
        Self([
            Self::new_plane(ctx, resolution.width, resolution.height),
            Self::new_plane(ctx, resolution.width / 2, resolution.height / 2),
            Self::new_plane(ctx, resolution.width / 2, resolution.height / 2),
        ])
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

    fn new_bind_group(&self, ctx: &WgpuCtx, layout: &wgpu::BindGroupLayout) -> wgpu::BindGroup {
        ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("yuv all textures bind group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.0[0].view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&self.0[1].view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&self.0[2].view),
                },
            ],
        })
    }
}

pub struct RGBATexture(pub Texture);

impl RGBATexture {
    pub fn new(ctx: &WgpuCtx, resolution: &Resolution) -> Self {
        Self(Texture::new(
            ctx,
            None,
            wgpu::Extent3d {
                width: resolution.width as u32,
                height: resolution.height as u32,
                depth_or_array_layers: 1,
            },
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::TEXTURE_BINDING,
        ))
    }

    pub fn new_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("single texture bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                ty: Texture::DEFAULT_BINDING_TYPE,
                visibility: wgpu::ShaderStages::FRAGMENT,
                count: None,
            }],
        })
    }

    fn new_bind_group(&self, ctx: &WgpuCtx, layout: &wgpu::BindGroupLayout) -> wgpu::BindGroup {
        ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("texture bind group"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&self.0.view),
            }],
        })
    }
}

pub struct InputTexture {
    pub textures: YUVTextures,
    pub bind_group: wgpu::BindGroup,
}

impl InputTexture {
    pub fn new(ctx: &WgpuCtx, resolution: &Resolution) -> Self {
        let textures = YUVTextures::new(ctx, resolution);
        let bind_group = textures.new_bind_group(ctx, &ctx.yuv_bind_group_layout);

        Self {
            textures,
            bind_group,
        }
    }
}

pub struct NodeTexture {
    pub texture: RGBATexture,
    pub bind_group: wgpu::BindGroup,
}

impl NodeTexture {
    pub fn new(ctx: &WgpuCtx, resolution: &Resolution) -> Self {
        let texture = RGBATexture::new(ctx, resolution);
        let bind_group = texture.new_bind_group(ctx, &ctx.rgba_bind_group_layout);

        Self {
            texture,
            bind_group,
        }
    }
}
pub struct OutputTexture {
    pub yuv_textures: YUVTextures,
    buffers: [wgpu::Buffer; 3],
    pub resolution: Resolution,
}

impl OutputTexture {
    fn padded(width: usize) -> usize {
        width + (256 - (width % 256))
    }
    pub fn new(ctx: &WgpuCtx, resolution: &Resolution) -> Self {
        let device = &ctx.device;
        let buffers = [
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("output texture buffer 0"),
                mapped_at_creation: false,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                size: (Self::padded(resolution.width) * resolution.height) as u64,
            }),
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("output texture buffer 1"),
                mapped_at_creation: false,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                size: (Self::padded(resolution.width / 2) * resolution.height / 2) as u64,
            }),
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("output texture buffer 2"),
                mapped_at_creation: false,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                size: (Self::padded(resolution.width / 2) * resolution.height / 2) as u64,
            }),
        ];
        Self {
            yuv_textures: YUVTextures::new(ctx, resolution),
            buffers,
            resolution: *resolution,
        }
    }

    /// extremely hacky workaround to download texture
    /// TODO: do it properly, potentially we might want to use DownloadBuffer
    pub fn download(&self, ctx: &WgpuCtx) -> YuvData {
        self.transfer_content_to_buffers(&ctx.device, &ctx.queue);
        let mut result = YuvData {
            y_plane: Bytes::new(),
            u_plane: Bytes::new(),
            v_plane: Bytes::new(),
        };
        {
            let size = self.yuv_textures.0[0].texture.size();
            let buffer = BytesMut::with_capacity(size.width as usize * size.height as usize);

            self.buffers[0]
                .slice(..)
                .map_async(MapMode::Read, |x| x.unwrap());
            ctx.device.poll(wgpu::MaintainBase::Wait);

            let range = self.buffers[0].slice(..).get_mapped_range();
            let chunks = range.chunks(Self::padded(size.width as usize));
            let mut buffer = buffer.writer();
            for chunk in chunks {
                buffer.write_all(&chunk[..size.width as usize]).unwrap();
            }
            result.y_plane = buffer.into_inner().into();
        }
        {
            let size = self.yuv_textures.0[1].texture.size();
            let buffer = BytesMut::with_capacity(size.width as usize * size.height as usize);

            self.buffers[1]
                .slice(..)
                .map_async(MapMode::Read, |x| x.unwrap());
            ctx.device.poll(wgpu::MaintainBase::Wait);

            let range = self.buffers[1].slice(..).get_mapped_range();
            let chunks = range.chunks(Self::padded(size.width as usize));
            let mut buffer = buffer.writer();
            for chunk in chunks {
                buffer.write_all(&chunk[..size.width as usize]).unwrap();
            }
            result.u_plane = buffer.into_inner().into();
        }
        {
            let size = self.yuv_textures.0[2].texture.size();
            let buffer = BytesMut::with_capacity(size.width as usize * size.height as usize);

            self.buffers[2]
                .slice(..)
                .map_async(MapMode::Read, |x| x.unwrap());
            ctx.device.poll(wgpu::MaintainBase::Wait);

            let range = self.buffers[2].slice(..).get_mapped_range();
            let chunks = range.chunks(Self::padded(size.width as usize));
            let mut buffer = buffer.writer();
            for chunk in chunks {
                buffer.write_all(&chunk[..size.width as usize]).unwrap();
            }
            result.v_plane = buffer.into_inner().into();
        }
        for buffer in self.buffers.iter() {
            buffer.unmap();
        }
        result
    }

    fn transfer_content_to_buffers(&self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("transfer result yuv texture to buffers encoder"),
        });

        for plane in [0, 1, 2] {
            encoder.copy_texture_to_buffer(
                wgpu::ImageCopyTexture {
                    aspect: wgpu::TextureAspect::All,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    texture: &self.yuv_textures.0[plane].texture,
                },
                wgpu::ImageCopyBuffer {
                    buffer: &self.buffers[plane],
                    layout: wgpu::ImageDataLayout {
                        bytes_per_row: Some(Self::padded(
                            self.yuv_textures.0[plane].texture.size().width as usize,
                        ) as u32),
                        rows_per_image: Some(self.yuv_textures.0[plane].texture.size().height),
                        offset: 0,
                    },
                },
                self.yuv_textures.0[plane].texture.size(),
            )
        }

        queue.submit(Some(encoder.finish()));
    }
}
