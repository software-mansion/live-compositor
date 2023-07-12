use std::{
    io::{Read, Write},
    net::{Ipv4Addr, TcpStream},
};

use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};

pub struct PacketStream(TcpStream);

impl PacketStream {
    pub fn connect(port: u16) -> Result<Self, PacketStreamError> {
        let stream = TcpStream::connect((Ipv4Addr::new(127, 0, 0, 1), port))?;
        Ok(Self(stream))
    }

    pub fn send_message(&mut self, msg: &[u8]) -> Result<(), PacketStreamError> {
        self.0
            .write_u32::<NetworkEndian>(msg.len() as u32)
            .map_err(PacketStreamError::SendFailure)?;
        self.0.write(msg).map_err(PacketStreamError::SendFailure)?;

        Ok(())
    }

    pub fn read_message(&mut self) -> Result<Vec<u8>, PacketStreamError> {
        let msg_len = self
            .0
            .read_u32::<NetworkEndian>()
            .map_err(PacketStreamError::ReadFailure)?;
        let mut data = vec![0; msg_len as usize];
        self.0
            .read_exact(&mut data)
            .map_err(PacketStreamError::ReadFailure)?;

        Ok(data)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PacketStreamError {
    #[error("failed to connect to web renderer")]
    ConnectionError(#[from] std::io::Error),

    #[error("failed to send packet")]
    SendFailure(std::io::Error),

    #[error("failed to read packet")]
    ReadFailure(std::io::Error),
}
