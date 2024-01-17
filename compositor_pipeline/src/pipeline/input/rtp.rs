use std::{
    net,
    sync::{atomic::AtomicBool, Arc},
    thread,
};

use crate::pipeline::{
    structs::{AudioCodec, EncodedChunk, EncodedChunkKind, VideoCodec},
    Port, RequestedPort,
};
use bytes::BytesMut;
use compositor_render::InputId;
use crossbeam_channel::{unbounded, Receiver, Sender};
use log::{error, warn};
use webrtc_util::Unmarshal;

use self::depayloader::Depayloader;

mod depayloader;

#[derive(Debug, thiserror::Error)]
pub enum RtpReceiverError {
    #[error("Error while setting socket options.")]
    SocketOptions(#[source] std::io::Error),

    #[error("Error while binding the socket.")]
    SocketBind(#[source] std::io::Error),

    #[error("Failed to register input. Port: {0} is already used or not available.")]
    PortAlreadyUsed(u16),

    #[error("Failed to register input. All ports in range {lower_bound} to {upper_bound} are already used or not available.")]
    AllPortsUsed { lower_bound: u16, upper_bound: u16 },
}

pub struct RtpReceiverOptions {
    pub port: RequestedPort,
    pub input_id: compositor_render::InputId,
    pub stream: RtpStream,
}

#[derive(Debug, Clone)]
pub struct VideoStream {
    pub codec: VideoCodec,
    pub payload_type: u8,
}

#[derive(Debug, Clone)]
pub struct AudioStream {
    pub codec: AudioCodec,
    pub payload_type: u8,
}

#[derive(Debug, Clone)]
pub struct RtpStream {
    pub video: Option<VideoStream>,
    pub audio: Option<AudioStream>,
}

pub struct RtpReceiver {
    receiver_thread: Option<thread::JoinHandle<()>>,
    should_close: Arc<AtomicBool>,
    pub port: u16,
}

impl RtpReceiver {
    pub fn new(opts: RtpReceiverOptions) -> Result<(Self, ChunksReceiver, Port), RtpReceiverError> {
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

        let port = match opts.port {
            RequestedPort::Exact(port) => {
                socket
                    .bind(
                        &net::SocketAddr::V4(net::SocketAddrV4::new(
                            net::Ipv4Addr::UNSPECIFIED,
                            port,
                        ))
                        .into(),
                    )
                    .map_err(|err| match err.kind() {
                        std::io::ErrorKind::AddrInUse => RtpReceiverError::PortAlreadyUsed(port),
                        _ => RtpReceiverError::SocketBind(err),
                    })?;
                port
            }
            RequestedPort::Range((lower_bound, upper_bound)) => {
                let port = (lower_bound..upper_bound).find(|port| {
                    let bind_res = socket.bind(
                        &net::SocketAddr::V4(net::SocketAddrV4::new(
                            net::Ipv4Addr::UNSPECIFIED,
                            *port,
                        ))
                        .into(),
                    );

                    bind_res.is_ok()
                });

                match port {
                    Some(port) => port,
                    None => {
                        return Err(RtpReceiverError::AllPortsUsed {
                            lower_bound,
                            upper_bound,
                        })
                    }
                }
            }
        };

        socket
            .set_read_timeout(Some(std::time::Duration::from_millis(50)))
            .map_err(RtpReceiverError::SocketOptions)?;

        let socket = std::net::UdpSocket::from(socket);

        let should_close2 = should_close.clone();

        let receiver_thread = thread::Builder::new()
            .name(format!("RTP receiver {}", opts.input_id))
            .spawn(move || RtpReceiver::rtp_receiver(socket, packets_tx, should_close2))
            .unwrap();

        let depayloader = Depayloader::new(&opts.stream);

        Ok((
            Self {
                port,
                receiver_thread: Some(receiver_thread),
                should_close,
            },
            ChunksReceiver::new(&opts.input_id, packets_rx, depayloader),
            Port(port),
        ))
    }

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

#[derive(Debug)]
pub struct ChunksReceiver {
    pub video: Option<Receiver<EncodedChunk>>,
    pub audio: Option<Receiver<EncodedChunk>>,
    should_close: Arc<AtomicBool>,
    depayloader_thread: Option<thread::JoinHandle<()>>,
}

impl ChunksReceiver {
    pub fn new(
        input_id: &InputId,
        receiver: Receiver<bytes::Bytes>,
        mut depayloader: Depayloader,
    ) -> Self {
        let should_close = Arc::new(AtomicBool::new(false));
        let (video_sender, video_receiver) = depayloader
            .video
            .as_ref()
            .map(|_| unbounded())
            .map_or((None, None), |(tx, rx)| (Some(tx), Some(rx)));
        let (audio_sender, audio_receiver) = depayloader
            .audio
            .as_ref()
            .map(|_| unbounded())
            .map_or((None, None), |(tx, rx)| (Some(tx), Some(rx)));

        let should_close2 = should_close.clone();
        let depayloader_thread = std::thread::Builder::new()
        .name(format!("Depayloading thread for input: {}", input_id.0))
        .spawn(move || {
            loop {
                if should_close2.load(std::sync::atomic::Ordering::Relaxed) {
                    return;
                }

                let mut buffer = receiver.recv().unwrap();

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
                        match depayloader.depayload(packet) {
                            Ok(Some(chunk)) => match &chunk.kind {
                                EncodedChunkKind::Video(_) => video_sender
                                    .as_ref()
                                    .map(|video_sender| video_sender.send(chunk)),
                                EncodedChunkKind::Audio(_) => audio_sender
                                    .as_ref()
                                    .map(|audio_sender| audio_sender.send(chunk)),
                            },
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
        })
        .unwrap();

        Self {
            video: video_receiver,
            audio: audio_receiver,
            should_close,
            depayloader_thread: Some(depayloader_thread),
        }
    }
}

impl Drop for ChunksReceiver {
    fn drop(&mut self) {
        self.should_close
            .store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(thread) = self.depayloader_thread.take() {
            thread.join().unwrap();
        } else {
            error!("RTP depayloader does not hold a thread handle to the receiving thread.")
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DepayloadingError {
    #[error("Bad payload type {0}")]
    BadPayloadType(u8),
    #[error(transparent)]
    Rtp(#[from] rtp::Error),
}
