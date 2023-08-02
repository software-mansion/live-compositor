use std::{
    io::Write,
    mem,
    ops::DerefMut,
    sync::{Arc, Mutex},
};

use bytes::{BufMut, Bytes, BytesMut};
use compositor_common::{scene::Resolution, Frame};
use crossbeam_channel::bounded;
use log::error;
use wgpu::{Buffer, BufferAsyncError, MapMode};

use self::{utils::pad_to_256, yuv::YUVPendingDownload};

use super::WgpuCtx;

mod base;
pub mod rgba;
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

impl InputTexture {
    pub fn new(ctx: &WgpuCtx, resolution: &Resolution) -> Self {
        let textures = YUVTextures::new(ctx, resolution);
        let bind_group = textures.new_bind_group(ctx, &ctx.yuv_bind_group_layout);

        Self {
            textures,
            bind_group,
            resolution: *resolution,
        }
    }

    pub fn upload(&mut self, ctx: &WgpuCtx, frame: Arc<Frame>) {
        if frame.resolution != self.resolution {
            self.textures = YUVTextures::new(ctx, &frame.resolution);
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
    pub fn new(ctx: &WgpuCtx, resolution: &Resolution) -> Self {
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
    pub fn new(ctx: &WgpuCtx, resolution: &Resolution) -> Self {
        Self {
            inner: InnerNodeTexture::new(ctx, resolution).into(),
            resolution: *resolution,
        }
    }

    pub fn ensure_size(&self, ctx: &WgpuCtx, resolution: &Resolution) {
        if *resolution != self.resolution {
            let new_inner = InnerNodeTexture::new(ctx, resolution);
            let mut guard = self.inner.lock().unwrap();
            let _ = mem::replace(guard.deref_mut(), new_inner);
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
    pub fn new(ctx: &WgpuCtx, resolution: &Resolution) -> Self {
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

    pub fn resolution(&self) -> &Resolution {
        &self.resolution
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
            self.download_buffer(self.textures.plains[0].texture.size(), &self.buffers[0]),
            self.download_buffer(self.textures.plains[1].texture.size(), &self.buffers[1]),
            self.download_buffer(self.textures.plains[2].texture.size(), &self.buffers[2]),
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
