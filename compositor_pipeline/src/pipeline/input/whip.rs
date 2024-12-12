use rand::{thread_rng, RngCore};
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use tokio::sync::mpsc;
use webrtc::track::track_remote::TrackRemote;

use depayloader::{Depayloader, DepayloaderNewError};
use std::fmt::Write;
use tracing::{error, warn};

use crate::{
    pipeline::{
        decoder::{self},
        encoder,
        rtp::BindToPortError,
        types::EncodedChunk,
        whip_whep::{WhipInputConnectionOptions, WhipWhepState},
        PipelineCtx,
    },
    queue::PipelineEvent,
};
use compositor_render::InputId;
use crossbeam_channel::Sender;
use tracing::{debug, span, Level};

use super::{AudioInputReceiver, Input, InputInitInfo, InputInitResult, VideoInputReceiver};

pub mod depayloader;

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

    #[error("Failed to add input {0} to input_connections hashmap")]
    WhipWhepStateAddInput(InputId),
}

#[derive(Debug, Clone)]
pub struct WhipReceiverOptions {
    pub video: Option<InputVideoStream>,
    pub audio: Option<InputAudioStream>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputVideoStream {
    pub options: decoder::VideoDecoderOptions,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputAudioStream {
    pub options: decoder::OpusDecoderOptions,
}

pub struct OutputAudioStream {
    pub options: encoder::EncoderOptions,
    pub payload_type: u8,
}

pub struct WhipReceiver {
    whip_whep_state: Arc<WhipWhepState>,
    input_id: InputId,
}

impl WhipReceiver {
    pub(super) fn start_new_input(
        input_id: &InputId,
        opts: WhipReceiverOptions,
        pipeline_ctx: &PipelineCtx,
    ) -> Result<InputInitResult, WhipReceiverError> {
        let bearer_token = generate_token();
        let whip_whep_state = pipeline_ctx.whip_whep_state.clone();

        let (mut video_tx, mut video_rx) = (None, None);
        let (mut video_tx_crossbeam, mut video_rx_crossbeam) = (None, None);

        if opts.video.is_some() {
            let (tx, rx) = mpsc::channel(100);
            video_tx = Some(tx);
            video_rx = Some(rx);
            let (tx_crossbeam, rx_crossbeam) = crossbeam_channel::bounded(100);
            video_tx_crossbeam = Some(tx_crossbeam);
            video_rx_crossbeam = Some(rx_crossbeam);
        }

        let (mut audio_tx, mut audio_rx) = (None, None);
        let (mut audio_tx_crossbeam, mut audio_rx_crossbeam) = (None, None);

        if opts.audio.is_some() {
            let (tx, rx) = mpsc::channel(100);
            audio_tx = Some(tx);
            audio_rx = Some(rx);
            let (tx_crossbeam, rx_crossbeam) = crossbeam_channel::bounded(100);
            audio_tx_crossbeam = Some(tx_crossbeam);
            audio_rx_crossbeam = Some(rx_crossbeam);
        }

        let depayloader = Arc::from(Mutex::new(Depayloader::new(&opts)?));

        Self::start_forwarding_thread(
            input_id,
            video_rx,
            audio_rx,
            video_tx_crossbeam,
            audio_tx_crossbeam,
        );

        if let Ok(mut input_connections) = whip_whep_state.input_connections.lock() {
            input_connections.insert(
                input_id.clone(),
                WhipInputConnectionOptions {
                    audio_sender: audio_tx.clone(),
                    video_sender: video_tx.clone(),
                    bearer_token: Some(bearer_token.clone()),
                    peer_connection: None,
                    start_time_vid: None,
                    start_time_aud: None,
                    depayloader,
                },
            );
        } else {
            return Err(WhipReceiverError::WhipWhepStateAddInput(input_id.clone()));
        }

        let video = match (video_rx_crossbeam, opts.video) {
            (Some(chunk_receiver), Some(stream)) => Some(VideoInputReceiver::Encoded {
                chunk_receiver,
                decoder_options: stream.options,
            }),
            _ => None,
        };
        let audio = match (audio_rx_crossbeam, opts.audio) {
            (Some(chunk_receiver), Some(stream)) => Some(AudioInputReceiver::Encoded {
                chunk_receiver,
                decoder_options: decoder::AudioDecoderOptions::Opus(stream.options),
            }),
            _ => None,
        };

        Ok(InputInitResult {
            input: Input::Whip(Self {
                whip_whep_state,
                input_id: input_id.clone(),
            }),
            video,
            audio,
            init_info: InputInitInfo::Whip { bearer_token },
        })
    }

    fn start_forwarding_thread(
        input_id: &InputId,
        video_mpsc_receiver: Option<mpsc::Receiver<PipelineEvent<EncodedChunk>>>,
        audio_mpsc_receiver: Option<mpsc::Receiver<PipelineEvent<EncodedChunk>>>,
        video_sender: Option<Sender<PipelineEvent<EncodedChunk>>>,
        audio_sender: Option<Sender<PipelineEvent<EncodedChunk>>>,
    ) {
        if let (Some(mut receiver), Some(sender)) = (video_mpsc_receiver, video_sender) {
            let input_id_clone = input_id.clone();
            thread::spawn(move || {
                let _span = span!(
                    Level::INFO,
                    "Forwarding Video",
                    input_id = input_id_clone.to_string()
                )
                .entered();
                loop {
                    let Some(chunk) = receiver.blocking_recv() else {
                        debug!("Closing RTP forwarding thread.");
                        break;
                    };

                    if let Err(err) = sender.send(chunk) {
                        debug!("Failed to send Encoded Chunk. Channel closed: {:?}", err);
                        break;
                    }
                }
            });
        }
        if let (Some(mut receiver), Some(sender)) = (audio_mpsc_receiver, audio_sender) {
            let input_id_clone = input_id.clone();
            thread::spawn(move || {
                let _span = span!(
                    Level::INFO,
                    "Forwarding Audio",
                    input_id = input_id_clone.to_string()
                )
                .entered();
                loop {
                    let Some(chunk) = receiver.blocking_recv() else {
                        debug!("Closing RTP forwarding thread.");
                        break;
                    };

                    if let Err(err) = sender.send(chunk) {
                        debug!("Failed to send Encoded Chunk. Channel closed: {:?}", err);
                        break;
                    }
                }
            });
        }
    }
}

impl Drop for WhipReceiver {
    fn drop(&mut self) {
        if let Ok(mut connections) = self.whip_whep_state.input_connections.lock() {
            if let Some(connection) = connections.get_mut(&self.input_id) {
                if let Some(peer_connection) = connection.peer_connection.clone() {
                    let input_id_clone = self.input_id.clone();
                    tokio::spawn(async move {
                        if let Err(err) = peer_connection.close().await {
                            error!(
                                "Cannot close peer_connection for {:?}: {:?}",
                                input_id_clone, err
                            );
                        };
                    });
                }
            }
            connections.remove(&self.input_id);
        }
    }
}

pub async fn handle_track(
    track: Arc<TrackRemote>,
    state: Arc<WhipWhepState>,
    input_id: InputId,
    depayloader: Arc<Mutex<Depayloader>>,
    sender: mpsc::Sender<PipelineEvent<EncodedChunk>>,
) {
    let mut first_pts_current_stream = None;
    let track_kind = track.kind();
    let state_clone = state.clone();

    let mut input_start_time = None;
    if let Ok(mut connections) = state_clone.input_connections.lock() {
        if let Some(connection) = connections.get_mut(&input_id) {
            if connection.set_start_time(track_kind).is_err() {
                error!("Cannot set start_time of audio/video stream");
            }
            input_start_time = connection.get_start_time(track_kind);
        } else {
            error!("InputID {input_id:?} not found");
        }
    } else {
        error!("Input connections lock error");
    }

    //TODO send PipelineEvent::NewPeerConnection to reset queue and decoder(drop remaining frames from previous stream)

    let mut first_chunk_flag = true;

    while let Ok((rtp_packet, _)) = track.read_rtp().await {
        let Ok(chunks) = depayloader.lock().unwrap().depayload(rtp_packet) else {
            warn!("RTP depayloading error",);
            continue;
        };
        if let Some(first_chunk) = chunks.first() {
            if first_chunk_flag {
                first_chunk_flag = false;
                first_pts_current_stream = Some(first_chunk.pts);
            }
        }

        for mut chunk in chunks {
            chunk.pts = chunk.pts + input_start_time.unwrap_or(Duration::ZERO)
                - first_pts_current_stream.unwrap_or(Duration::ZERO);
            if let Err(e) = sender.send(PipelineEvent::Data(chunk)).await {
                debug!("Failed to send audio RTP packet: {e}");
            }
        }
    }
}

fn generate_token() -> String {
    let mut bytes = [0u8; 16];
    thread_rng().fill_bytes(&mut bytes);
    bytes.iter().fold(String::new(), |mut acc, byte| {
        if let Err(err) = write!(acc, "{byte:02X}") {
            error!("Cannot generate token: {err:?}")
        }
        acc
    })
}

#[derive(Debug, thiserror::Error)]
pub enum DepayloadingError {
    #[error("Bad payload type {0}")]
    BadPayloadType(u8),
    #[error(transparent)]
    Rtp(#[from] rtp::Error),
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
