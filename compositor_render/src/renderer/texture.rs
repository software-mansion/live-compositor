use std::{
    io::Write,
    sync::{Arc, Mutex},
};

use bytes::{BufMut, Bytes, BytesMut};
use compositor_common::{scene::Resolution, Frame};
use crossbeam_channel::bounded;
use log::error;
use wgpu::{Buffer, BufferAsyncError, MapMode};

use self::{
    utils::{pad_to_256, texture_size_to_resolution},
    yuv::YUVPendingDownload,
};

use super::WgpuCtx;

mod base;
mod bgra;
mod rgba;
mod utils;
mod yuv;

pub type BGRATexture = bgra::BGRATexture;
pub type RGBATexture = rgba::RGBATexture;
pub type YUVTextures = yuv::YUVTextures;

pub type Texture = base::Texture;

pub struct InputTextureState {
    textures: YUVTextures,
    bind_group: wgpu::BindGroup,
}

impl InputTextureState {
    pub fn yuv_textures(&self) -> &YUVTextures {
        &self.textures
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn resolution(&self) -> Resolution {
        self.textures.resolution
    }
}

pub struct InputTexture {
    inner: Option<InputTextureState>,
    is_empty: bool,
}

impl InputTexture {
    pub fn new() -> Self {
        Self {
            inner: None,
            is_empty: true,
        }
    }

    pub fn clear(&mut self) {
        self.is_empty = true
    }

    pub fn upload(&mut self, ctx: &WgpuCtx, frame: Frame) {
        let inner = self.ensure_size(ctx, frame.resolution);
        inner.textures.upload(ctx, &frame.data)
    }

    fn ensure_size<'a>(
        &'a mut self,
        ctx: &WgpuCtx,
        new_resolution: Resolution,
    ) -> &'a InputTextureState {
        if let Some(inner) = self.inner.as_ref() {
            if inner.textures.resolution == new_resolution {
                self.is_empty = false;
                return self.inner.as_ref().unwrap();
            };
        };

        let textures = YUVTextures::new(ctx, new_resolution);
        let bind_group = textures.new_bind_group(ctx, ctx.format.yuv_layout());
        self.inner = Some(InputTextureState {
            textures,
            bind_group,
        });
        self.is_empty = false;
        self.inner.as_ref().unwrap()
    }

    pub fn state(&self) -> Option<&InputTextureState> {
        match self.is_empty {
            true => None,
            false => self.inner.as_ref(),
        }
    }
}

impl Default for InputTexture {
    fn default() -> Self {
        Self::new()
    }
}

/// Object representing current state of a NodeTexture.
/// This object represents temporary state and should be used
/// immediately after creation.
#[derive(Clone)]
pub struct NodeTextureState {
    texture: Arc<RGBATexture>,
    bind_group: Arc<wgpu::BindGroup>,
}

impl NodeTextureState {
    fn new(ctx: &WgpuCtx, resolution: Resolution) -> Self {
        let texture = RGBATexture::new(ctx, resolution);
        let bind_group = texture.new_bind_group(ctx, ctx.format.rgba_layout());

        Self {
            texture: Arc::new(texture),
            bind_group: Arc::new(bind_group),
        }
    }

    pub fn rgba_texture(&self) -> &RGBATexture {
        &self.texture
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn resolution(&self) -> Resolution {
        texture_size_to_resolution(&self.texture.size())
    }
}

struct InnerNodeTexture {
    state: Option<NodeTextureState>,
    is_empty: bool,
}

pub struct NodeTexture {
    inner: Mutex<InnerNodeTexture>,
}

impl NodeTexture {
    pub fn new() -> Self {
        Self {
            inner: InnerNodeTexture {
                state: None,
                is_empty: true,
            }
            .into(),
        }
    }

    pub fn clear(&self) {
        self.inner.lock().unwrap().is_empty = true
    }

    pub fn ensure_size(&self, ctx: &WgpuCtx, new_resolution: Resolution) -> NodeTextureState {
        let mut guard = self.inner.lock().unwrap();
        if let Some(ref state) = guard.state {
            if texture_size_to_resolution(&state.texture.size()) == new_resolution {
                let state = state.clone();
                guard.is_empty = false;
                return state;
            };
        };
        let new_inner = NodeTextureState::new(ctx, new_resolution);
        guard.state = Some(new_inner.clone());
        guard.is_empty = false;
        new_inner
    }

    pub fn state(&self) -> Option<NodeTextureState> {
        let guard = self.inner.lock().unwrap();
        match guard.is_empty {
            true => None,
            false => guard.state.clone(),
        }
    }

    pub fn resolution(&self) -> Option<Resolution> {
        let guard = self.inner.lock().unwrap();
        match guard.is_empty {
            true => None,
            false => guard.state.as_ref().map(NodeTextureState::resolution),
        }
    }
}

impl Default for NodeTexture {
    fn default() -> Self {
        Self::new()
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
