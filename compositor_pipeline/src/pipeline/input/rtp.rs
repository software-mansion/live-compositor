use std::{
    sync::{atomic::AtomicBool, Arc},
    thread,
};

use crate::{
    pipeline::{
        decoder::{self, DecoderOptions},
        encoder,
        rtp::{bind_to_requested_port, BindToPortError, RequestedPort, TransportProtocol},
        structs::EncodedChunkKind,
        Port,
    },
    queue::PipelineEvent,
};
use compositor_render::InputId;
use crossbeam_channel::{unbounded, Receiver};
use tracing::{debug, error, info, warn};
use webrtc_util::Unmarshal;

use self::{
    depayloader::{Depayloader, DepayloaderNewError},
    tcp_server::run_tcp_server_receiver,
    udp::run_udp_receiver,
};

use super::ChunksReceiver;

mod depayloader;
mod tcp_server;
mod udp;

#[derive(Debug, thiserror::Error)]
pub enum RtpReceiverError {
    #[error("Error while setting socket options.")]
    SocketOptions(#[source] std::io::Error),

    #[error("Error while binding the socket.")]
    SocketBind(#[source] std::io::Error),

    #[error("Failed to register input. Port: {0} is already used or not available.")]
    PortAlreadyInUse(u16),

    #[error("Failed to register input. All ports in range {lower_bound} to {upper_bound} are already used or not available.")]
    AllPortsAlreadyInUse { lower_bound: u16, upper_bound: u16 },

    #[error(transparent)]
    DepayloaderError(#[from] DepayloaderNewError),
}

pub struct RtpReceiverOptions {
    pub port: RequestedPort,
    pub transport_protocol: TransportProtocol,
    pub input_id: compositor_render::InputId,
    pub stream: RtpStream,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputVideoStream {
    pub options: decoder::VideoDecoderOptions,
    pub payload_type: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputAudioStream {
    pub options: decoder::AudioDecoderOptions,
    pub payload_type: u8,
}

pub struct OutputAudioStream {
    pub options: encoder::EncoderOptions,
    pub payload_type: u8,
}

#[derive(Debug, Clone)]
pub struct RtpStream {
    pub video: Option<InputVideoStream>,
    pub audio: Option<InputAudioStream>,
}

pub struct RtpReceiver {
    should_close: Arc<AtomicBool>,
    pub port: u16,
}

impl RtpReceiver {
    pub fn new(
        opts: RtpReceiverOptions,
    ) -> Result<(Self, ChunksReceiver, DecoderOptions, Port), RtpReceiverError> {
        let should_close = Arc::new(AtomicBool::new(false));

        let (port, packets_rx) = match opts.transport_protocol {
            TransportProtocol::Udp => Self::start_udp_reader(&opts, should_close.clone())?,
            TransportProtocol::TcpServer => {
                Self::start_tcp_server_reader(&opts, should_close.clone())?
            }
        };

        let depayloader = Depayloader::new(&opts.stream)?;

        let chunks_receiver = start_depayloader_thread(&opts.input_id, packets_rx, depayloader);

        Ok((
            Self {
                port: port.0,
                should_close,
            },
            chunks_receiver,
            DecoderOptions {
                video: opts.stream.video.map(|v| v.options),
                audio: opts.stream.audio.map(|a| a.options),
            },
            port,
        ))
    }

    fn start_tcp_server_reader(
        opts: &RtpReceiverOptions,
        should_close: Arc<AtomicBool>,
    ) -> Result<(Port, Receiver<bytes::Bytes>), RtpReceiverError> {
        let (packets_tx, packets_rx) = unbounded();
        let input_id = opts.input_id.clone();
        info!(input_id=?input_id.0, "Starting tcp socket");

        let socket = socket2::Socket::new(
            socket2::Domain::IPV4,
            socket2::Type::STREAM,
            Some(socket2::Protocol::TCP),
        )
        .map_err(RtpReceiverError::SocketOptions)?;

        let port = bind_to_requested_port(opts.port, &socket)?;

        socket.listen(1).map_err(RtpReceiverError::SocketBind)?;

        let socket = std::net::TcpListener::from(socket);

        thread::Builder::new()
            .name(format!("RTP TCP server receiver {}", opts.input_id))
            .spawn(move || {
                run_tcp_server_receiver(socket, packets_tx, should_close);
                debug!(input_id=?input_id.0, "Closing RTP receiver thread (TCP server).");
            })
            .unwrap();

        Ok((port, packets_rx))
    }

    fn start_udp_reader(
        opts: &RtpReceiverOptions,
        should_close: Arc<AtomicBool>,
    ) -> Result<(Port, Receiver<bytes::Bytes>), RtpReceiverError> {
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
                warn!("Failed to set socket receive buffer size: {e} This may cause packet loss, especially on high-bitrate streams.");
            }
        }

        let port = bind_to_requested_port(opts.port, &socket)?;

        socket
            .set_read_timeout(Some(std::time::Duration::from_millis(50)))
            .map_err(RtpReceiverError::SocketOptions)?;

        let socket = std::net::UdpSocket::from(socket);

        let input_id = opts.input_id.clone();
        thread::Builder::new()
            .name(format!("RTP UDP receiver {}", opts.input_id))
            .spawn(move || {
                run_udp_receiver(socket, packets_tx, should_close);
                debug!(input_id=?input_id.0, "Closing RTP receiver thread (UDP).");
            })
            .unwrap();

        Ok((port, packets_rx))
    }
}

impl Drop for RtpReceiver {
    fn drop(&mut self) {
        self.should_close
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

fn start_depayloader_thread(
    input_id: &InputId,
    receiver: Receiver<bytes::Bytes>,
    mut depayloader: Depayloader,
) -> ChunksReceiver {
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

    let input_id = input_id.clone();
    std::thread::Builder::new()
            .name(format!("Depayloading thread for input: {}", input_id.0))
            .spawn(move || {
                loop {
                    let Ok(mut buffer) = receiver.recv() else {
                        debug!(input_id=?input_id.0, "Closing RTP depayloader thread.");
                        return;
                    };

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
                                        .map(|video_sender| video_sender.send(PipelineEvent::Data(chunk))),
                                    EncodedChunkKind::Audio(_) => audio_sender
                                        .as_ref()
                                        .map(|audio_sender| audio_sender.send(PipelineEvent::Data(chunk))),
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

    ChunksReceiver {
        video: video_receiver,
        audio: audio_receiver,
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DepayloadingError {
    #[error("Bad payload type {0}")]
    BadPayloadType(u8),
    #[error(transparent)]
    Rtp(#[from] rtp::Error),
}

impl From<BindToPortError> for RtpReceiverError {
    fn from(value: BindToPortError) -> Self {
        match value {
            BindToPortError::SocketBind(err) => RtpReceiverError::SocketBind(err),
            BindToPortError::PortAlreadyInUse(port) => RtpReceiverError::PortAlreadyInUse(port),
            BindToPortError::AllPortsAlreadyInUse {
                lower_bound,
                upper_bound,
            } => RtpReceiverError::AllPortsAlreadyInUse {
                lower_bound,
                upper_bound,
            },
        }
    }
}
