use std::net::TcpStream;

use super::{
    packet::{Packet, PacketError},
    Url,
};

#[derive(Debug)]
pub enum Command<'a> {
    Use(Url<'a>),
    Resolution { width: u32, height: u32 },
    // TODO: Implement
    // Source {
    //     name: &'a str,
    //     buffer: &'a [u8]
    // },
    Render,
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
