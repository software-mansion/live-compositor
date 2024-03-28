use std::{
    io::Read,
    net::{Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};

use crate::common::CommunicationProtocol;
use anyhow::{Context, Result};
use bytes::{Bytes, BytesMut};
use crossbeam_channel::Receiver;
use tracing::info;

pub struct OutputReceiver {
    receiver: Receiver<Bytes>,
    dump_length: Duration,
}

impl OutputReceiver {
    pub fn start<P: AsRef<Path>>(
        port: u16,
        protocol: CommunicationProtocol,
        dump_length: Duration,
        dump_path: P,
    ) -> Result<Self> {
        let mut socket = Self::setup_socket(port, &protocol)?;
        let mut output_dump = BytesMut::new();
        let mut buffer = BytesMut::zeroed(u16::MAX as usize);
        let mut start = None;
        let dump_path = dump_path.as_ref().to_path_buf();
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
                if start.elapsed() > dump_length {
                    break;
                }
            }

            if cfg!(feature = "update_snapshots") {
                info!("Updating output dump: {dump_path:?}");
                let save_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .parent()
                    .unwrap()
                    .join("snapshot_tests")
                    .join("snapshots")
                    .join("rtp_packet_dumps")
                    .join("outputs")
                    .join(dump_path);
                std::fs::write(save_path, &output_dump)
                    .context("Failed to write output dump")
                    .unwrap();
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
            .recv_timeout(self.dump_length + Duration::from_secs(10))
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
