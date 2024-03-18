use anyhow::{Context, Result};
use bytes::{Bytes, BytesMut};
use crossbeam_channel::Receiver;
use rtp::packet::Packet;
use std::{
    fs,
    io::Read,
    net::{Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};
use tracing::info;
use webrtc_util::Unmarshal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommunicationProtocol {
    Udp,
    Tcp,
}

pub struct OutputReceiver {
    receiver: Receiver<Bytes>,
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
        })
    }

    pub fn recv(self) -> Result<Bytes> {
        self.receiver
            .recv()
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

pub fn input_dump_from_disk<P: AsRef<Path>>(path: P) -> Result<Bytes> {
    let input_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("snapshot_tests")
        .join("snapshots")
        .join("rtp_packet_dumps")
        .join("inputs")
        .join(path);

    let bytes = fs::read(input_path).context("Failed to read input dump")?;
    Ok(Bytes::from(bytes))
}

pub fn output_dump_from_disk<P: AsRef<Path>>(path: P) -> Result<Bytes> {
    let output_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("snapshot_tests")
        .join("snapshots")
        .join("rtp_packet_dumps")
        .join("outputs")
        .join(path);

    let bytes = fs::read(output_path).context("Failed to read output dump")?;
    Ok(Bytes::from(bytes))
}

pub fn compare_dumps(
    expected: &Bytes,
    actual: &Bytes,
    time_range: (Duration, Duration),
) -> Result<()> {
    fn unmarshal_packets(data: &Bytes) -> Result<Vec<Packet>> {
        let mut packets = Vec::new();
        let mut read_bytes = 0;
        while read_bytes < data.len() {
            let packet_size = u16::from_be_bytes([data[read_bytes], data[read_bytes + 1]]) as usize;
            read_bytes += 2;

            if data.len() < read_bytes + packet_size {
                break;
            }

            // TODO(noituri): Goodbye packet
            let packet = Packet::unmarshal(&mut &data[read_bytes..(read_bytes + packet_size)])?;
            read_bytes += packet_size;

            packets.push(packet);
        }

        Ok(packets)
    }

    fn find_packets_in_range(
        mut packets: Vec<Packet>,
        range: (Duration, Duration),
        clock_rate: u32,
    ) -> Result<Vec<Packet>> {
        let start_timestamp = (range.0.as_secs_f32() * clock_rate as f32) as u32;
        let end_timestamp = (range.1.as_secs_f32() * clock_rate as f32) as u32;

        packets.retain(|p| {
            let timestamp = p.header.timestamp;
            timestamp >= start_timestamp && timestamp <= end_timestamp
        });

        Ok(packets)
    }

    let expected_packets = unmarshal_packets(expected)?;
    let actual_packets = unmarshal_packets(actual)?;

    let clock_rate = expected_packets
        .first()
        .map(|p| match p.header.payload_type {
            // Video
            96 => 90000,
            // Audio Opus
            97 => 48000,
            _ => unreachable!("Unsupported payload type"),
        })
        .ok_or_else(|| anyhow::anyhow!("No packets found"))?;

    let expected_packets = find_packets_in_range(expected_packets.clone(), time_range, clock_rate)?;
    let actual_packets = find_packets_in_range(actual_packets.clone(), time_range, clock_rate)?;
    for (expected_packet, actual_packet) in expected_packets.iter().zip(actual_packets.iter()) {
        validate_packets(expected_packet, actual_packet)?;
    }

    Ok(())
}

// TODO(noituri): Implement this properly
fn validate_packets(expected: &Packet, actual: &Packet) -> Result<()> {
    if expected.header.payload_type != actual.header.payload_type {
        return Err(anyhow::anyhow!("Payload type mismatch"));
    }

    Ok(())
}
