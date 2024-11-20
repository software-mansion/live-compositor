use rand::Rng;
use std::{
    sync::{Arc, Mutex},
    thread,
};
use tokio::sync::mpsc;

use depayloader::{Depayloader, DepayloaderNewError};
use tracing::error;

use std::sync::atomic::AtomicBool;

use crate::{
    pipeline::{
        decoder::{self},
        encoder,
        rtp::BindToPortError,
        types::EncodedChunk,
        whip_whep::InputConnectionUtils,
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
}

#[derive(Debug, Clone)]
pub struct WhipReceiverOptions {
    pub stream: WhipStream,
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

#[derive(Debug, Clone)]
pub struct WhipStream {
    pub video: Option<InputVideoStream>,
    pub audio: Option<InputAudioStream>,
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

        let bearer_token = generate_token();

        let whip_whep_state = pipeline_ctx.whip_whep_state.clone();

        let (mut video_tx, mut video_rx) = (None, None);
        let (mut video_tx_crossbeam, mut video_rx_crossbeam) = (None, None);

        if opts.stream.video.is_some() {
            let (tx, rx) = mpsc::channel(100);
            video_tx = Some(tx);
            video_rx = Some(rx);

            let (tx_crossbeam, rx_crossbeam) = crossbeam_channel::bounded(100);
            video_tx_crossbeam = Some(tx_crossbeam);
            video_rx_crossbeam = Some(rx_crossbeam);
        }

        let (mut audio_tx, mut audio_rx) = (None, None);
        let (mut audio_tx_crossbeam, mut audio_rx_crossbeam) = (None, None);

        if opts.stream.audio.is_some() {
            let (tx, rx) = mpsc::channel(100);
            audio_tx = Some(tx);
            audio_rx = Some(rx);

            let (tx_crossbeam, rx_crossbeam) = crossbeam_channel::bounded(100);
            audio_tx_crossbeam = Some(tx_crossbeam);
            audio_rx_crossbeam = Some(rx_crossbeam);
        }

        let depayloader = Arc::from(Mutex::new(Depayloader::new(&opts.stream)?));

        Self::start_forwarding_thread(
            input_id,
            video_rx,
            audio_rx,
            video_tx_crossbeam,
            audio_tx_crossbeam,
        );

        whip_whep_state.input_connections.lock().unwrap().insert(
            input_id.clone(),
            InputConnectionUtils {
                audio_sender: audio_tx.clone(),
                video_sender: video_tx.clone(),
                bearer_token: Some(bearer_token.clone()),
                peer_connection: None,
                start_time_vid: None,
                start_time_aud: None,
                depayloader,
            },
        );

        let video = match (video_rx_crossbeam, opts.stream.video) {
            (Some(chunk_receiver), Some(stream)) => Some(VideoInputReceiver::Encoded {
                chunk_receiver,
                decoder_options: stream.options,
            }),
            _ => None,
        };
        let audio = match (audio_rx_crossbeam, opts.stream.audio) {
            (Some(chunk_receiver), Some(stream)) => Some(AudioInputReceiver::Encoded {
                chunk_receiver,
                decoder_options: decoder::AudioDecoderOptions::Opus(stream.options),
            }),
            _ => None,
        };

        Ok(InputInitResult {
            input: Input::Whip(Self { should_close }),
            video,
            audio,
            init_info: InputInitInfo::BearerToken(bearer_token),
        })
    }

    fn start_forwarding_thread(
        input_id: &InputId,
        video_mpsc_receiver: Option<tokio::sync::mpsc::Receiver<PipelineEvent<EncodedChunk>>>,
        audio_mpsc_receiver: Option<tokio::sync::mpsc::Receiver<PipelineEvent<EncodedChunk>>>,
        video_sender: Option<Sender<PipelineEvent<EncodedChunk>>>,
        audio_sender: Option<Sender<PipelineEvent<EncodedChunk>>>,
    ) {
        if let (Some(receiver), Some(sender)) = (video_mpsc_receiver, video_sender) {
            let input_id_clone = input_id.clone();
            thread::spawn(move || {
                let _span = span!(
                    Level::INFO,
                    "Mid Audio",
                    input_id = input_id_clone.to_string()
                )
                .entered();
                run_forwarding_loop(receiver, sender)
            });
        }
        if let (Some(receiver), Some(sender)) = (audio_mpsc_receiver, audio_sender) {
            let input_id_clone = input_id.clone();
            thread::spawn(move || {
                let _span = span!(
                    Level::INFO,
                    "Mid Audio",
                    input_id = input_id_clone.to_string()
                )
                .entered();
                run_forwarding_loop(receiver, sender)
            });
        }
    }
}

impl Drop for WhipReceiver {
    fn drop(&mut self) {
        self.should_close
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

fn run_forwarding_loop(
    mut receiver: tokio::sync::mpsc::Receiver<PipelineEvent<EncodedChunk>>,
    sender: Sender<PipelineEvent<EncodedChunk>>,
) {
    loop {
        let Some(chunk) = receiver.blocking_recv() else {
            debug!("Closing RTP depayloader thread.");
            break;
        };

        let _ = sender.send(chunk);
    }
}

fn generate_token() -> String {
    let mut rng = rand::thread_rng();
    (0..17)
        .map(|_| format!("{:02x}", rng.gen::<u8>()))
        .collect()
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
