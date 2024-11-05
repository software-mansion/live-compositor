use axum::extract::State;
use bytes::Bytes;
use rtp::packet::Packet;
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
    thread,
};
use tokio::sync::mpsc;

use depayloader::{Depayloader, DepayloaderNewError};
use serde_json::{json, Value};
use tracing::{error, info};

use std::sync::atomic::AtomicBool;

use crate::{
    pipeline::{
        decoder::{self},
        encoder,
        rtp::BindToPortError,
        types::{EncodedChunk, EncodedChunkKind},
        whip_whep::InputConnectionUtils,
        PipelineCtx,
    },
    queue::PipelineEvent,
};
use compositor_render::InputId;
use crossbeam_channel::{bounded, Receiver, Sender};
use rtcp::header::PacketType;
use tracing::{debug, span, warn, Level};
use webrtc_util::Unmarshal;

use super::{AudioInputReceiver, Input, InputInitInfo, InputInitResult, VideoInputReceiver};

mod depayloader;

#[derive(Debug, thiserror::Error)]
pub enum WhipReceiverError {
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

#[derive(Debug, Clone)]
pub struct WhipReceiverOptions {
    pub bearer_token: String,
    pub stream: WhipStream,
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
pub struct WhipStream {
    pub video: Option<InputVideoStream>,
    pub audio: Option<InputAudioStream>,
}

struct DepayloaderThreadReceivers {
    video: Option<Receiver<PipelineEvent<EncodedChunk>>>,
    audio: Option<Receiver<PipelineEvent<EncodedChunk>>>,
}
pub struct WhipReceiver {
    should_close: Arc<AtomicBool>,
}

impl WhipReceiver {
    pub(super) fn start_new_input(
        input_id: &InputId,
        opts: WhipReceiverOptions,
        pipeline_ctx: &PipelineCtx,
    ) -> Result<InputInitResult, WhipReceiverError> {
        let should_close = Arc::new(AtomicBool::new(false));

        let whip_whep_state = pipeline_ctx.whip_whep_state.clone();

        let (video_tx, video_rx) = mpsc::channel(100);
        let (audio_tx, audio_rx) = mpsc::channel(100);

        info!("{:?}", video_rx.is_closed());
        info!("{:?}", audio_rx.is_closed());

        whip_whep_state.input_connections.lock().unwrap().insert(
            input_id.clone(),
            InputConnectionUtils {
                audio_receiver: None,
                audio_sender: Some(audio_tx.clone()),
                video_receiver: None,
                video_sender: Some(video_tx.clone()),
                bearer_token: None,
                peer_connection: None,
            },
        );
        info!("Added to hashmap: {:?}", whip_whep_state.input_connections);

        let depayloader = Arc::from(Mutex::new(Depayloader::new(&opts.stream)?));

        let depayloader_receivers =
            Self::start_depayloader_thread(input_id, video_rx, audio_rx, depayloader);

        let video = match (depayloader_receivers.video, opts.stream.video) {
            (Some(chunk_receiver), Some(stream)) => Some(VideoInputReceiver::Encoded {
                chunk_receiver,
                decoder_options: stream.options,
            }),
            _ => None,
        };
        let audio = match (depayloader_receivers.audio, opts.stream.audio) {
            (Some(chunk_receiver), Some(stream)) => Some(AudioInputReceiver::Encoded {
                chunk_receiver,
                decoder_options: stream.options,
            }),
            _ => None,
        };

        Ok(InputInitResult {
            input: Input::Whip(Self { should_close }),
            video,
            audio,
            init_info: InputInitInfo { port: None },
        })
    }

    fn start_depayloader_thread(
        input_id: &InputId,
        video_mpsc_receiver: tokio::sync::mpsc::Receiver<Packet>,
        audio_mpsc_receiver: tokio::sync::mpsc::Receiver<Packet>,
        depayloader: Arc<Mutex<Depayloader>>,
    ) -> DepayloaderThreadReceivers {
        let (video_sender, video_receiver) = depayloader
            .lock()
            .unwrap()
            .video
            .as_ref()
            .map(|_| bounded(5))
            .map_or((None, None), |(tx, rx)| (Some(tx), Some(rx)));

        let (audio_sender, audio_receiver) = depayloader
            .lock()
            .unwrap()
            .audio
            .as_ref()
            .map(|_| bounded(5))
            .map_or((None, None), |(tx, rx)| (Some(tx), Some(rx)));

        let depayloader_clone = depayloader.clone();
        let input_id_clone = input_id.clone();
        thread::spawn(move || {
            let _span = span!(
                Level::INFO,
                "RTP depayloader video",
                input_id = input_id_clone.to_string()
            )
            .entered();
            run_depayloader_loop(video_mpsc_receiver, depayloader_clone, video_sender.clone())
        });

        let depayloader_clone = depayloader.clone();
        let input_id_clone = input_id.clone();
        thread::spawn(move || {
            let _span = span!(
                Level::INFO,
                "RTP depayloader audio",
                input_id = input_id_clone.to_string()
            )
            .entered();
            run_depayloader_loop(audio_mpsc_receiver, depayloader_clone, audio_sender.clone())
        });

        DepayloaderThreadReceivers {
            video: video_receiver,
            audio: audio_receiver,
        }
    }
}

impl Drop for WhipReceiver {
    fn drop(&mut self) {
        self.should_close
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

fn run_depayloader_loop(
    mut receiver: tokio::sync::mpsc::Receiver<Packet>,
    depayloader: Arc<Mutex<Depayloader>>,
    sender: Option<Sender<PipelineEvent<EncodedChunk>>>,
) {
    let sender_clone = sender.clone();
    let mut eos_received = sender.as_ref().map(|_| false);
    let mut ssrc = None;

    let mut maybe_send_eos = || {
        if let (Some(sender), Some(false)) = (&sender, eos_received) {
            eos_received = Some(true);
            if sender.send(PipelineEvent::EOS).is_err() {
                debug!("Failed to send EOS from RTP video depayloader. Channel closed.");
            }
        }
    };
    loop {
        let Some(packet) = receiver.blocking_recv() else {
            debug!("Closing RTP depayloader thread.");
            break;
        };

        // match rtp::packet::Packet::unmarshal(&mut buffer.clone()) {
        // https://datatracker.ietf.org/doc/html/rfc5761#section-4
        //
        // Given these constraints, it is RECOMMENDED to follow the guidelines
        // in the RTP/AVP profile [7] for the choice of RTP payload type values,
        // with the additional restriction that payload type values in the range
        // 64-95 MUST NOT be used.
        if packet.header.payload_type < 64 || packet.header.payload_type > 95 {
            if ssrc.is_none() {
                ssrc = Some(packet.header.ssrc);
            }
            // info!("{:?}", packet.header.payload_type);

            match depayloader.lock().unwrap().depayload(packet) {
                Ok(chunks) => {
                    for chunk in chunks {
                        // println!("{:?}", chunk);
                        sender_clone
                            .as_ref()
                            .map(|sender_clone| sender_clone.send(PipelineEvent::Data(chunk)));
                    }
                }
                Err(err) => {
                    warn!("RTP depayloading error: {}", err);
                    continue;
                }
            }
        }

        // Ok(_) | Err(_) => {
        //     match rtcp::packet::unmarshal(&mut buffer) {
        //         Ok(rtcp_packets) => {
        // for rtcp_packet in rtcp_packets {
        //     if let PacketType::Goodbye = rtcp_packet.header().packet_type {
        //         for ssrc in rtcp_packet.destination_ssrc() {
        //             if Some(ssrc) == audio_ssrc {
        //                 maybe_send_audio_eos()
        //             }
        //             if Some(ssrc) == video_ssrc {
        //                 maybe_send_video_eos()
        //             }
        //         }
        //     } else {
        //         debug!(
        //             packet_type=?rtcp_packet.header().packet_type,
        //             "Received RTCP packet"
        //         )
        //     }
        // }
        //         }
        //         Err(err) => {
        //             warn!(%err, "Received an unexpected packet, which is not recognized either as RTP or RTCP. Dropping.");
        //         }
        //     }
        //     continue;
        // }
    }
    // }
    maybe_send_eos();
}

#[derive(Debug, thiserror::Error)]
pub enum DepayloadingError {
    #[error("Bad payload type {0}")]
    BadPayloadType(u8),
    #[error(transparent)]
    Rtp(#[from] rtp::Error),
    #[error("AAC depayoading error")]
    Aac(#[from] depayloader::AacDepayloadingError),
}

impl From<BindToPortError> for WhipReceiverError {
    fn from(value: BindToPortError) -> Self {
        match value {
            BindToPortError::SocketBind(err) => WhipReceiverError::SocketBind(err),
            BindToPortError::PortAlreadyInUse(port) => WhipReceiverError::PortAlreadyInUse(port),
            BindToPortError::AllPortsAlreadyInUse {
                lower_bound,
                upper_bound,
            } => WhipReceiverError::AllPortsAlreadyInUse {
                lower_bound,
                upper_bound,
            },
        }
    }
}
