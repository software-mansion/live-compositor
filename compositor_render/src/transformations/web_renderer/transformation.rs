use std::{cell::RefCell, collections::HashMap, error::Error, rc::Rc};

use compositor_common::scene::{Resolution, TransformationRegistryKey};
use image::ImageError;

use crate::renderer::{
    texture::Texture,
    transformation::{Transformation, TransformationParams},
    WgpuCtx,
};

use super::{
    command::Command,
    packet_stream::{PacketStream, PacketStreamError},
    Url,
};

pub struct WebRenderer {
    wgpu_ctx: Rc<WgpuCtx>,
    stream: RefCell<PacketStream>,
}

impl Transformation for WebRenderer {
    fn apply(
        &self,
        params: &TransformationParams,
        _sources: &HashMap<String, Texture>,
        target: &Texture,
    ) -> Result<(), Box<dyn Error>> {
        let TransformationParams::String(url) = params else {
            return Err(Box::new(WebRendererApplyError::InvalidParam));
        };

        let frame = self.request_frame(url, target.size())?;
        if !frame.is_empty() {
            self.upload(&frame, target)?;
        }

        Ok(())
    }

    fn registry_key(&self) -> TransformationRegistryKey {
        TransformationRegistryKey("web_renderer".to_owned())
    }
}

impl WebRenderer {
    pub fn new(wgpu_ctx: Rc<WgpuCtx>, port: u16) -> Result<Self, WebRendererNewError> {
        let stream = PacketStream::connect(port)?;

        Ok(Self {
            wgpu_ctx,
            stream: RefCell::new(stream),
        })
    }

    fn upload(&self, data: &[u8], target: &Texture) -> Result<(), WebRendererApplyError> {
        let size = target.size();
        let img = image::load_from_memory(data)?;

        self.wgpu_ctx.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &target.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &img.to_rgba8(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * size.width),
                rows_per_image: Some(size.height),
            },
            size,
        );

        self.wgpu_ctx.queue.submit([]);
        Ok(())
    }

    fn request_frame(
        &self,
        url: Url<'_>,
        size: wgpu::Extent3d,
    ) -> Result<Vec<u8>, WebRendererApplyError> {
        let mut stream = self.stream.borrow_mut();
        let commands = [
            Command::Use(url),
            Command::Resolution(Resolution {
                width: size.width as usize,
                height: size.height as usize,
            }),
            Command::Render,
        ];

        for cmd in commands {
            stream.send_message(&cmd.get_message())?;
        }

        let frame = stream.read_message()?;
        Ok(frame)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WebRendererNewError {
    #[error("failed to establish connection with web renderer")]
    ConnectionFailure(#[from] PacketStreamError),
}

#[derive(Debug, thiserror::Error)]
pub enum WebRendererApplyError {
    #[error("expected string param")]
    InvalidParam,

    #[error("communication with web renderer failed")]
    PacketError(#[from] PacketStreamError),

    #[error("failed to decode image data")]
    ImageDecodeError(#[from] ImageError),
}
