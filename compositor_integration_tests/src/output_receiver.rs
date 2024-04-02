use std::{
    io::Read,
    net::{Ipv4Addr, SocketAddr},
    thread,
    time::{Duration, Instant},
};

use crate::common::CommunicationProtocol;
use anyhow::{Context, Result};
use bytes::{Bytes, BytesMut};
use crossbeam_channel::Receiver;

pub struct OutputReceiver {
    receiver: Receiver<Bytes>,
    dump_length: Duration,
}

impl OutputReceiver {
    pub fn start(
        port: u16,
        protocol: CommunicationProtocol,
        dump_length: Duration,
    ) -> Result<Self> {
        let mut socket = Self::setup_socket(port, &protocol)?;
        let mut output_dump = BytesMut::new();
        let mut buffer = BytesMut::zeroed(u16::MAX as usize);
        let mut start = None;
        let (dump_sender, dump_receiver) = crossbeam_channel::bounded(1);

        thread::spawn(move || {
            loop {
                let received_bytes = socket.read(&mut buffer).unwrap();
                let start = start.get_or_insert_with(Instant::now);

                if protocol == CommunicationProtocol::Udp {
                    let packet_len_bytes = (received_bytes as u16).to_be_bytes();
                    output_dump.extend_from_slice(&packet_len_bytes);
                }

                output_dump.extend_from_slice(&buffer[..received_bytes]);
                // TODO(noituri): This does not work on slower machines which take longer time to process video and audio
                // It results in shorter output dumps than expected
                if start.elapsed() > dump_length {
                    break;
                }
            }

            dump_sender.send(output_dump.freeze()).unwrap();
        });

        Ok(Self {
            receiver: dump_receiver,
            dump_length,
        })
    }

    pub fn wait_for_output(self) -> Result<Bytes> {
        self.receiver
            .recv_timeout(self.dump_length + Duration::from_secs(60))
            .context("Failed to receive output dump")
    }

    fn setup_socket(port: u16, protocol: &CommunicationProtocol) -> Result<socket2::Socket> {
        let socket = match protocol {
            CommunicationProtocol::Udp => socket2::Socket::new(
                socket2::Domain::IPV4,
                socket2::Type::DGRAM,
                Some(socket2::Protocol::UDP),
            )?,
            CommunicationProtocol::Tcp => socket2::Socket::new(
                socket2::Domain::IPV4,
                socket2::Type::STREAM,
                Some(socket2::Protocol::TCP),
            )?,
        };

        match protocol {
            CommunicationProtocol::Udp => {
                socket.bind(&SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), port).into())?;
            }
            CommunicationProtocol::Tcp => {
                socket
                    .connect(&SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), port).into())?;
            }
        }

        Ok(socket)
    }
}
