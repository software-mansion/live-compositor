use std::{
    net,
    sync::{atomic::AtomicBool, Arc},
    thread,
};

use crate::pipeline::structs::{
    AudioChannels, AudioCodec, EncodedChunk, EncodedChunkKind, VideoCodec,
};
use bytes::BytesMut;
use crossbeam_channel::{unbounded, Receiver, Sender};
use log::{error, warn};
use webrtc_util::Unmarshal;

use self::depayloader::Depayloader;

mod depayloader;

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
    pub stream: RtpStream,
}

#[derive(Debug, Clone)]
pub enum RtpStream {
    Video(VideoCodec),
    Audio {
        codec: AudioCodec,
        sample_rate: u32,
        channels: AudioChannels,
    },
    VideoWithAudio {
        video_codec: VideoCodec,
        video_payload_type: u8,
        audio_codec: AudioCodec,
        audio_payload_type: u8,
        audio_channels: AudioChannels,
    },
}

pub enum ChunksReceiver {
    Video(Receiver<EncodedChunk>),
    Audio(Receiver<EncodedChunk>),
    VideoWithAudio {
        video: Receiver<EncodedChunk>,
        audio: Receiver<EncodedChunk>,
    },
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

        let depayloader = Depayloader::new(&opts.stream);

        Ok((
            Self {
                port: opts.port,
                receiver_thread: Some(receiver_thread),
                should_close,
            },
            ChunkIter::new(packets_rx, depayloader),
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
pub enum ChunkIter {
    Video(Receiver<EncodedChunk>),
    Audio(Receiver<EncodedChunk>),
    VideoWithAudio {
        video: Receiver<EncodedChunk>,
        audio: Receiver<EncodedChunk>,
    },
}

impl ChunkIter {
    pub fn new(receiver: Receiver<bytes::Bytes>, mut depayloader: Depayloader) -> Self {
        let (chunk_iter, video_sender, audio_sender) = match &depayloader {
            Depayloader::Video(_) => {
                let (video_sender, video_receiver) = unbounded();
                (ChunkIter::Video(video_receiver), Some(video_sender), None)
            }
            Depayloader::Audio(_) => {
                let (audio_sender, audio_receiver) = unbounded();
                (ChunkIter::Audio(audio_receiver), None, Some(audio_sender))
            }
            Depayloader::VideoWithAudio { .. } => {
                let (video_sender, video_receiver) = unbounded();
                let (audio_sender, audio_receiver) = unbounded();
                (
                    ChunkIter::VideoWithAudio {
                        video: video_receiver,
                        audio: audio_receiver,
                    },
                    Some(video_sender),
                    Some(audio_sender),
                )
            }
        };

        std::thread::spawn(move || {
            loop {
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
        });

        chunk_iter
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DepayloadingError {
    #[error("Bad payload type {0}")]
    BadPayloadType(u8),
    #[error(transparent)]
    Rtp(#[from] rtp::Error),
}
