use std::{
    io::Write,
    sync::{Arc, Mutex},
};

use bytes::{BufMut, Bytes, BytesMut};
use compositor_common::{scene::Resolution, util::RGBColor, Frame};
use crossbeam_channel::bounded;
use log::error;
use wgpu::{Buffer, BufferAsyncError, MapMode};

use self::{utils::pad_to_256, yuv::YUVPendingDownload};

use super::WgpuCtx;

mod base;
mod rgba;
mod utils;
mod yuv;

pub type RGBATexture = rgba::RGBATexture;
pub type YUVTextures = yuv::YUVTextures;

pub type Texture = base::Texture;

pub struct InputTexture {
    textures: YUVTextures,
    bind_group: wgpu::BindGroup,
    resolution: Resolution,
}

fn rgb_to_yuv(rgb: RGBColor) -> (f32, f32, f32) {
    let r = rgb.0 as f32 / 255.0;
    let g = rgb.1 as f32 / 255.0;
    let b = rgb.2 as f32 / 255.0;
    (
        ((0.299 * r) + (0.587 * g) + (0.114 * b)).clamp(0.0, 1.0),
        (((-0.168736 * r) - (0.331264 * g) + (0.5 * b)) + (128.0 / 255.0)).clamp(0.0, 1.0),
        (((0.5 * r) + (-0.418688 * g) + (-0.081312 * b)) + (128.0 / 255.0)).clamp(0.0, 1.0),
    )
}

impl InputTexture {
    pub fn new(ctx: &WgpuCtx, resolution: Resolution, init_color: Option<RGBColor>) -> Self {
        let textures = YUVTextures::new(ctx, resolution);
        let bind_group = textures.new_bind_group(ctx, &ctx.yuv_bind_group_layout);

        let (y, u, v) = rgb_to_yuv(init_color.unwrap_or(RGBColor(0, 0, 0)));
        ctx.r8_fill_with_color_pipeline
            .fill(ctx, textures.plane(0), y);
        ctx.r8_fill_with_color_pipeline
            .fill(ctx, textures.plane(1), u);
        ctx.r8_fill_with_color_pipeline
            .fill(ctx, textures.plane(2), v);

        Self {
            textures,
            bind_group,
            resolution,
        }
    }

    pub fn upload(&mut self, ctx: &WgpuCtx, frame: Frame) {
        if frame.resolution != self.resolution {
            self.textures = YUVTextures::new(ctx, frame.resolution);
            self.bind_group = self
                .textures
                .new_bind_group(ctx, &ctx.yuv_bind_group_layout);
            self.resolution = frame.resolution;
        }
        self.textures.upload(ctx, &frame.data)
    }

    pub fn yuv_textures(&self) -> &YUVTextures {
        &self.textures
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

struct InnerNodeTexture {
    texture: Arc<RGBATexture>,
    bind_group: Arc<wgpu::BindGroup>,
}

impl InnerNodeTexture {
    pub fn new(ctx: &WgpuCtx, resolution: Resolution) -> Self {
        let texture = RGBATexture::new(ctx, resolution);
        let bind_group = texture.new_bind_group(ctx, &ctx.rgba_bind_group_layout);

        Self {
            texture: Arc::new(texture),
            bind_group: Arc::new(bind_group),
        }
    }
}

pub struct NodeTexture {
    inner: Mutex<InnerNodeTexture>,
    pub resolution: Resolution,
}

impl NodeTexture {
    pub fn new(ctx: &WgpuCtx, resolution: Resolution) -> Self {
        Self {
            inner: InnerNodeTexture::new(ctx, resolution).into(),
            resolution,
        }
    }

    pub fn ensure_size(&self, ctx: &WgpuCtx, resolution: Resolution) {
        if resolution != self.resolution {
            let new_inner = InnerNodeTexture::new(ctx, resolution);
            let mut guard = self.inner.lock().unwrap();
            *guard = new_inner;
        }
    }

    pub fn rgba_texture(&self) -> Arc<RGBATexture> {
        let guard = self.inner.lock().unwrap();
        guard.texture.clone()
    }

    pub fn bind_group(&self) -> Arc<wgpu::BindGroup> {
        let guard = self.inner.lock().unwrap();
        guard.bind_group.clone()
    }
}

pub struct OutputTexture {
    textures: YUVTextures,
    buffers: [wgpu::Buffer; 3],
    resolution: Resolution,
}

impl OutputTexture {
    pub fn new(ctx: &WgpuCtx, resolution: Resolution) -> Self {
        let textures = YUVTextures::new(ctx, resolution);
        let buffers = textures.new_download_buffers(ctx);
        Self {
            textures,
            buffers,
            resolution: resolution.to_owned(),
        }
    }

    pub fn yuv_textures(&self) -> &YUVTextures {
        &self.textures
    }

    pub fn resolution(&self) -> Resolution {
        self.resolution
    }

    pub fn start_download<'a>(
        &'a self,
        ctx: &WgpuCtx,
    ) -> YUVPendingDownload<
        'a,
        impl FnOnce() -> Result<Bytes, BufferAsyncError> + 'a,
        BufferAsyncError,
    > {
        self.textures.copy_to_buffers(ctx, &self.buffers);

        YUVPendingDownload::new(
            self.download_buffer(self.textures.planes[0].texture.size(), &self.buffers[0]),
            self.download_buffer(self.textures.planes[1].texture.size(), &self.buffers[1]),
            self.download_buffer(self.textures.planes[2].texture.size(), &self.buffers[2]),
        )
    }

    fn download_buffer<'a>(
        &'a self,
        size: wgpu::Extent3d,
        source: &'a Buffer,
    ) -> impl FnOnce() -> Result<Bytes, BufferAsyncError> + 'a {
        let buffer = BytesMut::with_capacity((size.width * size.height) as usize);
        let (s, r) = bounded(1);
        source.slice(..).map_async(MapMode::Read, move |result| {
            if let Err(err) = s.send(result) {
                error!("channel send error: {err}")
            }
        });

        move || {
            r.recv().unwrap()?;
            let mut buffer = buffer.writer();
            {
                let range = source.slice(..).get_mapped_range();
                let chunks = range.chunks(pad_to_256(size.width) as usize);
                for chunk in chunks {
                    buffer.write_all(&chunk[..size.width as usize]).unwrap();
                }
            };
            source.unmap();
            Ok(buffer.into_inner().into())
        }
    }
}
