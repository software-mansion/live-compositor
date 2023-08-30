use std::{io::Write, mem};

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
pub mod utils;
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

pub struct InputTexture(OptionalState<InputTextureState>);

impl InputTexture {
    pub fn new() -> Self {
        Self(OptionalState::new())
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }

    pub fn upload(&mut self, ctx: &WgpuCtx, frame: Frame) {
        let state = self.ensure_size(ctx, frame.resolution);
        state.textures.upload(ctx, &frame.data)
    }

    fn ensure_size<'a>(
        &'a mut self,
        ctx: &WgpuCtx,
        new_resolution: Resolution,
    ) -> &'a InputTextureState {
        fn new_state(ctx: &WgpuCtx, new_resolution: Resolution) -> InputTextureState {
            let textures = YUVTextures::new(ctx, new_resolution);
            let bind_group = textures.new_bind_group(ctx, ctx.format.yuv_layout());
            InputTextureState {
                textures,
                bind_group,
            }
        }

        self.0 = match self.0.replace(OptionalState::None) {
            OptionalState::Some(state) | OptionalState::NoneWithOldState(state) => {
                if state.textures.resolution == new_resolution {
                    OptionalState::Some(state)
                } else {
                    OptionalState::Some(new_state(ctx, new_resolution))
                }
            }
            OptionalState::None => OptionalState::Some(new_state(ctx, new_resolution)),
        };
        self.state().unwrap()
    }

    pub fn state(&self) -> Option<&InputTextureState> {
        self.0.state()
    }
}

impl Default for InputTexture {
    fn default() -> Self {
        Self::new()
    }
}

pub struct NodeTextureState {
    texture: RGBATexture,
    bind_group: wgpu::BindGroup,
}

impl NodeTextureState {
    fn new(ctx: &WgpuCtx, resolution: Resolution) -> Self {
        let texture = RGBATexture::new(ctx, resolution);
        let bind_group = texture.new_bind_group(ctx, ctx.format.rgba_layout());

        Self {
            texture,
            bind_group,
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

pub struct NodeTexture(OptionalState<NodeTextureState>);

impl NodeTexture {
    pub fn new() -> Self {
        Self(OptionalState::new())
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }

    pub fn ensure_size<'a>(
        &'a mut self,
        ctx: &WgpuCtx,
        new_resolution: Resolution,
    ) -> &'a NodeTextureState {
        self.0 = match self.0.replace(OptionalState::None) {
            OptionalState::NoneWithOldState(state) | OptionalState::Some(state) => {
                if texture_size_to_resolution(&state.texture.size()) == new_resolution {
                    OptionalState::Some(state)
                } else {
                    let new_inner = NodeTextureState::new(ctx, new_resolution);
                    OptionalState::Some(new_inner)
                }
            }
            OptionalState::None => {
                let new_inner = NodeTextureState::new(ctx, new_resolution);
                OptionalState::Some(new_inner)
            }
        };
        self.0.state().unwrap().clone()
    }

    pub fn state(&self) -> Option<&NodeTextureState> {
        self.0.state()
    }

    pub fn resolution(&self) -> Option<Resolution> {
        self.0.state().map(NodeTextureState::resolution)
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

/// Type that behaves like Option, but when is set to None
/// it keeps ownership of the value it had before.
enum OptionalState<State> {
    None,
    /// It should be treated as None, but hold on the old state, so
    /// it can be reused in the future.
    NoneWithOldState(State),
    Some(State),
}

impl<State> OptionalState<State> {
    fn new() -> Self {
        Self::None
    }

    fn clear(&mut self) {
        *self = match self.replace(Self::None) {
            Self::None => Self::None,
            Self::NoneWithOldState(state) => Self::NoneWithOldState(state),
            Self::Some(state) => Self::NoneWithOldState(state),
        }
    }

    fn state(&self) -> Option<&State> {
        match self {
            OptionalState::None => None,
            OptionalState::NoneWithOldState(_) => None,
            OptionalState::Some(ref state) => Some(state),
        }
    }

    fn replace(&mut self, replacement: Self) -> Self {
        mem::replace(self, replacement)
    }
}

impl<State> Default for OptionalState<State> {
    fn default() -> Self {
        Self::None
    }
}
