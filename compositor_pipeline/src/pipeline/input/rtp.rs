use std::sync::{atomic::AtomicBool, Arc};

use crate::{
    pipeline::{
        decoder::{self, DecoderOptions},
        encoder,
        rtp::{BindToPortError, RequestedPort, TransportProtocol},
        structs::{EncodedChunk, EncodedChunkKind},
        Port,
    },
    queue::PipelineEvent,
};
use compositor_render::InputId;
use crossbeam_channel::{bounded, Receiver, Sender};
use tracing::{debug, error, span, warn, Level};
use webrtc_util::Unmarshal;

use self::{
    depayloader::{Depayloader, DepayloaderNewError},
    tcp_server::start_tcp_server_thread,
    udp::start_udp_reader_thread,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputAudioStream {
    pub options: decoder::AudioDecoderOptions,
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
            TransportProtocol::Udp => start_udp_reader_thread(&opts, should_close.clone())?,
            TransportProtocol::TcpServer => start_tcp_server_thread(&opts, should_close.clone())?,
        };

        let depayloader = Depayloader::new(&opts.stream)?;

        let chunks_receiver =
            Self::start_depayloader_thread(&opts.input_id, packets_rx, depayloader);

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

    fn start_depayloader_thread(
        input_id: &InputId,
        receiver: Receiver<bytes::Bytes>,
        depayloader: Depayloader,
    ) -> ChunksReceiver {
        let (video_sender, video_receiver) = depayloader
            .video
            .as_ref()
            .map(|_| bounded(5))
            .map_or((None, None), |(tx, rx)| (Some(tx), Some(rx)));
        let (audio_sender, audio_receiver) = depayloader
            .audio
            .as_ref()
            .map(|_| bounded(5))
            .map_or((None, None), |(tx, rx)| (Some(tx), Some(rx)));

        let input_id = input_id.clone();
        std::thread::Builder::new()
            .name(format!("Depayloading thread for input: {}", input_id.0))
            .spawn(move || {
                let _span = span!(
                    Level::INFO,
                    "RTP depayloader",
                    input_id = input_id.to_string()
                )
                .entered();
                run_depayloader_thread(receiver, depayloader, video_sender, audio_sender)
            })
            .unwrap();

        ChunksReceiver {
            video: video_receiver,
            audio: audio_receiver,
        }
    }
}

impl Drop for RtpReceiver {
    fn drop(&mut self) {
        self.should_close
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

fn run_depayloader_thread(
    receiver: Receiver<bytes::Bytes>,
    mut depayloader: Depayloader,
    video_sender: Option<Sender<PipelineEvent<EncodedChunk>>>,
    audio_sender: Option<Sender<PipelineEvent<EncodedChunk>>>,
) {
    loop {
        let Ok(mut buffer) = receiver.recv() else {
            debug!("Closing RTP depayloader thread.");
            break;
        };

        match rtp::packet::Packet::unmarshal(&mut buffer.clone()) {
            // https://datatracker.ietf.org/doc/html/rfc5761#section-4
            //
            // Given these constraints, it is RECOMMENDED to follow the guidelines
            // in the RTP/AVP profile [7] for the choice of RTP payload type values,
            // with the additional restriction that payload type values in the range
            // 64-95 MUST NOT be used.
            Ok(packet) if packet.header.payload_type < 64 || packet.header.payload_type > 95 => {
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
                // TODO: Handle RTCP Goodbye packet
                if rtcp::packet::unmarshal(&mut buffer).is_err() {
                    warn!("Received an unexpected packet, which is not recognized either as RTP or RTCP. Dropping.");
                }

                continue;
            }
        };
    }
    if let Some(sender) = video_sender {
        if let Err(_err) = sender.send(PipelineEvent::EOS) {
            debug!("Failed to send EOS from RTP video depayloader. Channel closed.");
        }
    }
    if let Some(sender) = audio_sender {
        if let Err(_err) = sender.send(PipelineEvent::EOS) {
            debug!("Failed to send EOS from RTP audio depayloader. Channel closed.");
        }
    };
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
