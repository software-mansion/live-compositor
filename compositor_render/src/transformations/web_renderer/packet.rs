use std::{net::TcpStream, io::{Write, Read}};

pub struct Packet<'a>(pub &'a [u8]);

impl<'a> Packet<'a> {
    pub fn send(&self, stream: &mut TcpStream) -> Result<(), PacketError> {
        let header = (self.0.len() as u32).to_be_bytes();
        let buffer = [&header, self.0].concat();
        stream.write(&buffer).map_err(PacketError::SendFailure)?;

        Ok(())
    }

    pub fn read(stream: &mut TcpStream) -> Result<Vec<u8>, PacketError> {
        let mut len = -1;
        let mut header = Vec::with_capacity(4);
        let mut msg = Vec::new();
        let mut buf = [0u8; 65535];
        let mut buf_len = 0;
        let mut buf_offset = 0;
    
        while msg.len() != len as usize {
            if buf_len <= buf_offset {
                buf_len = stream.read(&mut buf).map_err(PacketError::ReadFailure)?;
                buf_offset = 0;
            }
            if len == -1 {
                let n = 4 - header.len() + buf_offset;
                header.extend_from_slice(&buf[buf_offset..n]);
                buf_offset += n;
                if header.len() == 4 {
                    len = u32::from_be_bytes([header[0], header[1], header[2], header[3]]) as i32;
                    msg.reserve(len as usize);
                }
            }
            
            let n = (len as usize - msg.len() + buf_offset).min(buf_len);
            if msg.len() < len as usize {
                msg.extend_from_slice(&buf[buf_offset..n]);
                buf_offset += n;
            }
        }

        Ok(msg)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PacketError {
    #[error("failed to send packet")]
    SendFailure(std::io::Error),

    #[error("failed to read packet")]
    ReadFailure(std::io::Error),
}