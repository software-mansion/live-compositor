use std::{
    cell::RefCell,
    collections::HashMap,
    error::Error,
    net::{Ipv4Addr, TcpStream},
};

use compositor_common::scene::TransformationRegistryKey;

use crate::renderer::{
    texture::Texture,
    transformation::{Transformation, TransformationParams},
};

use super::{command::Command, packet::{PacketError, Packet}, Url};

pub struct WebRenderer {
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

        Ok(())
    }

    fn registry_key(&self) -> TransformationRegistryKey {
        TransformationRegistryKey("web_renderer".to_owned())
    }
}

impl WebRenderer {
    pub fn new(port: u16) -> Result<Self, WebRendererNewError> {
        let stream = TcpStream::connect((Ipv4Addr::new(127, 0, 0, 1), port))?;

        Ok(Self {
            stream: RefCell::new(stream),
        })
    }

    fn upload(data: &[u8], target: &Texture) {}

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
}
