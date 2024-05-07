use std::{
    io::Write,
    net::{Ipv4Addr, SocketAddr},
};

use anyhow::Result;

pub struct PacketSender {
    socket: socket2::Socket,
}

impl PacketSender {
    pub fn new(port: u16) -> Result<Self> {
        let socket = socket2::Socket::new(
            socket2::Domain::IPV4,
            socket2::Type::STREAM,
            Some(socket2::Protocol::TCP),
        )?;

        socket.connect(&SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), port).into())?;

        Ok(Self { socket })
    }

    pub fn send(&mut self, rtp_packets: &[u8]) -> Result<()> {
        self.socket.write_all(rtp_packets)?;
        Ok(())
    }
}
