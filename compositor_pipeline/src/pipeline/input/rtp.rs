use std::{
    net,
    sync::{atomic::AtomicBool, Arc},
    thread,
};

use crate::pipeline::structs::{Codec, EncodedChunk, EncodedChunkKind};
use bytes::BytesMut;
use crossbeam_channel::{unbounded, Receiver, Sender};
use log::{error, warn};
use rtp::{codecs::h264::H264Packet, packetizer::Depacketizer};
use webrtc_util::Unmarshal;

pub struct RtpReceiver {
    receiver_thread: Option<thread::JoinHandle<()>>,
    should_close: Arc<AtomicBool>,
    pub port: u16,
}

#[derive(Debug, thiserror::Error)]
pub enum RtpReceiverError {
    #[error("Error while setting socket options.")]
    SocketOptions(#[source] std::io::Error),

    #[error("Error while binding the socket.")]
    SocketBind(#[source] std::io::Error),
}

pub struct RtpReceiverOptions {
    pub port: u16,
    pub input_id: compositor_render::InputId,
}

impl RtpReceiver {
    pub fn new(opts: RtpReceiverOptions) -> Result<(Self, ChunkIter), RtpReceiverError> {
        let should_close = Arc::new(AtomicBool::new(false));
        let (packets_tx, packets_rx) = unbounded();

        let socket = socket2::Socket::new(
            socket2::Domain::IPV4,
            socket2::Type::DGRAM,
            Some(socket2::Protocol::UDP),
        )
        .map_err(RtpReceiverError::SocketOptions)?;

        match socket
            .set_recv_buffer_size(16 * 1024 * 1024)
            .map_err(RtpReceiverError::SocketOptions)
        {
            Ok(_) => {}
            Err(e) => {
                warn!("Failed to set socket receive buffer size: {e}. This may cause packet loss, especially on high-bitrate streams.");
            }
        }

        socket
            .bind(
                &net::SocketAddr::V4(net::SocketAddrV4::new(
                    net::Ipv4Addr::UNSPECIFIED,
                    opts.port,
                ))
                .into(),
            )
            .map_err(RtpReceiverError::SocketBind)?;

        socket
            .set_read_timeout(Some(std::time::Duration::from_millis(50)))
            .map_err(RtpReceiverError::SocketOptions)?;

        let socket = std::net::UdpSocket::from(socket);

        let should_close2 = should_close.clone();

        let receiver_thread = thread::Builder::new()
            .name(format!("RTP receiver {}", opts.input_id))
            .spawn(move || RtpReceiver::rtp_receiver(socket, packets_tx, should_close2))
            .unwrap();

        Ok((
            Self {
                port: opts.port,
                receiver_thread: Some(receiver_thread),
                should_close,
            },
            ChunkIter {
                receiver: packets_rx,
                depayloader: H264Packet::default(),
            },
        ))
    }
}

impl RtpReceiver {
    fn rtp_receiver(
        socket: std::net::UdpSocket,
        packets_tx: Sender<bytes::Bytes>,
        should_close: Arc<AtomicBool>,
    ) {
        let mut buffer = BytesMut::zeroed(65536);

        loop {
            if should_close.load(std::sync::atomic::Ordering::Relaxed) {
                return;
            }

            // This can be faster if we batched sending the packets through the channel
            let (received_bytes, _) = match socket.recv_from(&mut buffer) {
                Ok(n) => n,
                Err(e) => match e.kind() {
                    std::io::ErrorKind::WouldBlock => continue,
                    _ => {
                        error!("Error while receiving UDP packet: {}", e);
                        continue;
                    }
                },
            };

            let packet: bytes::Bytes = buffer[..received_bytes].to_vec().into();
            packets_tx.send(packet).unwrap();
        }
    }
}

impl Drop for RtpReceiver {
    fn drop(&mut self) {
        self.should_close
            .store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(thread) = self.receiver_thread.take() {
            thread.join().unwrap();
        } else {
            error!("RTP receiver does not hold a thread handle to the receiving thread.")
        }
    }
}

pub struct ChunkIter {
    receiver: Receiver<bytes::Bytes>,
    depayloader: H264Packet,
}

impl Iterator for ChunkIter {
    type Item = EncodedChunk;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let mut buffer = self.receiver.recv().ok()?;

            match rtp::packet::Packet::unmarshal(&mut buffer.clone()) {
                // https://datatracker.ietf.org/doc/html/rfc5761#section-4
                //
                // Given these constraints, it is RECOMMENDED to follow the guidelines
                // in the RTP/AVP profile [7] for the choice of RTP payload type values,
                // with the additional restriction that payload type values in the range
                // 64-95 MUST NOT be used.
                Ok(packet)
                    if packet.header.payload_type < 64 || packet.header.payload_type > 95 =>
                {
                    match chunk_from_rtp(packet, &mut self.depayloader) {
                        Ok(Some(chunk)) => return Some(chunk),
                        Ok(None) => continue,
                        Err(err) => {
                            warn!("RTP depayloading error: {}", err);
                            continue;
                        }
                    }
                }
                Ok(_) | Err(_) => {
                    if rtcp::packet::unmarshal(&mut buffer).is_err() {
                        warn!("Received an unexpected packet, which is not recognized either as RTP or RTCP. Dropping.");
                    }

                    continue;
                }
            };
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum DepayloadingError {
    #[error("Bad payload type {0}")]
    BadPayloadType(u8),
    #[error(transparent)]
    Rtp(#[from] rtp::Error),
}

fn chunk_from_rtp(
    packet: rtp::packet::Packet,
    depayloader: &mut H264Packet,
) -> Result<Option<EncodedChunk>, DepayloadingError> {
    match packet.header.payload_type {
        96 => {
            let kind = EncodedChunkKind::Video(Codec::H264);

            let h264_packet = depayloader.depacketize(&packet.payload)?;

            if h264_packet.is_empty() {
                return Ok(None);
            }

            Ok(Some(EncodedChunk {
                data: h264_packet,
                pts: packet.header.timestamp as i64,
                dts: None,

                kind,
            }))
        }

        v => Err(DepayloadingError::BadPayloadType(v)),
    }
}
