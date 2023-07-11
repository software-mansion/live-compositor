use std::{
    cell::RefCell,
    collections::HashMap,
    error::Error,
    net::{Ipv4Addr, TcpStream},
    rc::Rc,
};

use compositor_common::scene::TransformationRegistryKey;
use image::ImageError;

use crate::renderer::{
    texture::Texture,
    transformation::{Transformation, TransformationParams},
    WgpuCtx,
};

use super::{
    command::Command,
    packet::{Packet, PacketError},
    Url,
};

pub struct WebRenderer {
    wgpu_ctx: Rc<WgpuCtx>,
    stream: RefCell<TcpStream>,
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
        self.upload(&frame, target)?;
        // println!("FRAME: {}", frame.len());
        // std::fs::write("test.jpeg", frame)?;

        Ok(())
    }

    fn registry_key(&self) -> TransformationRegistryKey {
        TransformationRegistryKey("web_renderer".to_owned())
    }
}

impl WebRenderer {
    pub fn new(wgpu_ctx: Rc<WgpuCtx>, port: u16) -> Result<Self, WebRendererNewError> {
        let stream = TcpStream::connect((Ipv4Addr::new(127, 0, 0, 1), port))?;

        Ok(Self {
            wgpu_ctx,
            stream: RefCell::new(stream),
        })
    }

    fn upload(&self, data: &[u8], target: &Texture) -> Result<(), WebRendererApplyError> {
        let size = target.size();
        let img = image::load_from_memory(data)?;
        let data = img.to_rgba8();
        println!("IMG SIZE: {}", data.len());

        self.wgpu_ctx.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &target.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &data,
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
            Command::Resolution {
                width: size.width,
                height: size.height,
            },
            Command::Render,
        ];

        for cmd in commands {
            cmd.exec(&mut stream)?;
        }

        let frame = Packet::read(&mut stream)?;
        Ok(frame)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WebRendererNewError {
    #[error("failed to connect to web renderer")]
    ConnectionFailure(#[from] std::io::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum WebRendererApplyError {
    #[error("expected string param")]
    InvalidParam,

    #[error("communication with web renderer failed")]
    PacketError(#[from] PacketError),

    #[error("failed to decode image data")]
    ImageDecodeError(#[from] ImageError),
}
