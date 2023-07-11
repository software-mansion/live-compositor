use std::net::TcpStream;

use super::{Url, packet::{Packet, PacketError}};

#[derive(Debug)]
pub enum Command<'a> {
    Use(Url<'a>),
    Resolution {
        width: u32,
        height: u32
    },
    // TODO: Implement 
    // Source {}
    Render
}

impl<'a> Command<'a> {
    pub fn exec(&self, stream: &mut TcpStream) -> Result<(), PacketError> {
        let msg = match self {
            Command::Use(url) => format!("use:{url}"),
            Command::Resolution { width, height } => format!("resolution:{width}x{height}"),
            Command::Render => "render".to_owned(),
        };

        Packet(msg.as_bytes()).send(stream)
    }
}