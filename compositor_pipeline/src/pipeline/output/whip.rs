use compositor_render::OutputId;
use crossbeam_channel::{Receiver, Sender};
use establish_peer_connection::connect;
use init_peer_connection::init_peer_connection;
use packet_stream::PacketStream;
use payloader::{Payload, Payloader, PayloadingError};
use reqwest::{Method, StatusCode};
use std::{
    sync::{atomic::AtomicBool, Arc},
    thread,
    time::{Duration, Instant},
};
use tokio::{runtime::Runtime, sync::oneshot};
use tracing::{debug, error, span, Instrument, Level};
use url::{ParseError, Url};
use webrtc::track::track_local::TrackLocalWriter;

use crate::{
    error::OutputInitError,
    event::{Event, EventEmitter},
    pipeline::{AudioCodec, EncoderOutputEvent, PipelineCtx, VideoCodec},
};

mod establish_peer_connection;
mod init_peer_connection;
mod packet_stream;
mod payloader;

#[derive(Debug)]
pub struct WhipSender {
    pub connection_options: WhipSenderOptions,
    should_close: Arc<AtomicBool>,
}

#[derive(Debug, Clone)]
pub struct WhipSenderOptions {
    pub endpoint_url: String,
    pub bearer_token: Option<Arc<str>>,
    pub video: Option<VideoCodec>,
    pub audio: Option<AudioCodec>,
}

#[derive(Debug, Clone)]
pub struct WhipCtx {
    options: WhipSenderOptions,
    output_id: OutputId,
    request_keyframe_sender: Option<Sender<()>>,
    should_close: Arc<AtomicBool>,
    tokio_rt: Arc<Runtime>,
}

#[derive(Debug, thiserror::Error)]
pub enum WhipError {
    #[error("Bad status in WHIP response\nStatus: {0}\nBody: {1}")]
    BadStatus(StatusCode, String),

    #[error("WHIP request failed!\nMethod: {0}\nURL: {1}")]
    RequestFailed(Method, Url),

    #[error(
        "Unable to get location endpoint, check correctness of WHIP endpoint and your Bearer token"
    )]
    MissingLocationHeader,

    #[error("Invalid endpoint URL: {1}")]
    InvalidEndpointUrl(#[source] ParseError, String),

    #[error("Missing Host in endpoint URL")]
    MissingHost,

    #[error("Missing port in endpoint URL")]
    MissingPort,

    #[error("Failed to create RTC session description: {0}")]
    RTCSessionDescriptionError(webrtc::Error),

    #[error("Failed to set local description: {0}")]
    LocalDescriptionError(webrtc::Error),

    #[error("Failed to set remote description: {0}")]
    RemoteDescriptionError(webrtc::Error),

    #[error("Failed to parse {0} response body: {1}")]
    BodyParsingError(&'static str, reqwest::Error),

    #[error("Failed to create offer: {0}")]
    OfferCreationError(webrtc::Error),

    #[error(transparent)]
    PeerConnectionInitError(#[from] webrtc::Error),

    #[error("Failed to convert ICE candidate to JSON: {0}")]
    IceCandidateToJsonError(webrtc::Error),

    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),

    #[error(transparent)]
    PayloadingError(#[from] PayloadingError),

    #[error("Trickle ICE not supported")]
    TrickleIceNotSupported,

    #[error("Entity Tag missing")]
    EntityTagMissing,

    #[error("Entity Tag non-matching")]
    EntityTagNonMatching,
}

const WHIP_INIT_TIMEOUT: Duration = Duration::from_secs(60);

impl WhipSender {
    pub fn new(
        output_id: &OutputId,
        options: WhipSenderOptions,
        packets_receiver: Receiver<EncoderOutputEvent>,
        request_keyframe_sender: Option<Sender<()>>,
        pipeline_ctx: &PipelineCtx,
    ) -> Result<Self, OutputInitError> {
        let payloader = Payloader::new(options.video, options.audio);
        let packet_stream = PacketStream::new(packets_receiver, payloader, 1400);
        let should_close = Arc::new(AtomicBool::new(false));
        let (init_confirmation_sender, mut init_confirmation_receiver) =
            oneshot::channel::<Result<(), WhipError>>();

        let whip_ctx = WhipCtx {
            options: options.clone(),
            output_id: output_id.clone(),
            request_keyframe_sender,
            should_close: should_close.clone(),
            tokio_rt: pipeline_ctx.tokio_rt.clone(),
        };

        pipeline_ctx.tokio_rt.spawn(
            run_whip_sender_task(
                whip_ctx,
                packet_stream,
                init_confirmation_sender,
                pipeline_ctx.event_emitter.clone(),
            )
            .instrument(span!(
                Level::INFO,
                "WHIP sender",
                output_id = output_id.to_string()
            )),
        );

        let start_time = Instant::now();
        loop {
            thread::sleep(Duration::from_millis(500));
            let elapsed_time = Instant::now().duration_since(start_time);
            if elapsed_time > WHIP_INIT_TIMEOUT {
                return Err(OutputInitError::WhipInitTimeout);
            }
            match init_confirmation_receiver.try_recv() {
                Ok(result) => match result {
                    Ok(_) => break,
                    Err(err) => return Err(OutputInitError::WhipInitError(err.into())),
                },
                Err(err) => match err {
                    oneshot::error::TryRecvError::Closed => {
                        return Err(OutputInitError::UnknownWhipError)
                    }
                    oneshot::error::TryRecvError::Empty => {}
                },
            };
        }

        Ok(Self {
            connection_options: options,
            should_close,
        })
    }
}

impl Drop for WhipSender {
    fn drop(&mut self) {
        self.should_close
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

async fn run_whip_sender_task(
    whip_ctx: WhipCtx,
    packet_stream: PacketStream,
    init_confirmation_sender: oneshot::Sender<Result<(), WhipError>>,
    event_emitter: Arc<EventEmitter>,
) {
    let client = Arc::new(reqwest::Client::new());
    let (peer_connection, video_track, audio_track) = match init_peer_connection().await {
        Ok(pc) => pc,
        Err(err) => {
            init_confirmation_sender.send(Err(err)).unwrap();
            return;
        }
    };
    let output_id_clone = whip_ctx.output_id.clone();
    let should_close_clone = whip_ctx.should_close.clone();
    let whip_session_url = match connect(peer_connection, client.clone(), &whip_ctx).await {
        Ok(val) => val,
        Err(err) => {
            init_confirmation_sender.send(Err(err)).unwrap();
            return;
        }
    };
    init_confirmation_sender.send(Ok(())).unwrap();

    for chunk in packet_stream {
        if should_close_clone.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }
        let chunk = match chunk {
            Ok(chunk) => chunk,
            Err(err) => {
                error!("Failed to payload a packet: {}", err);
                continue;
            }
        };

        match chunk {
            Payload::Video(video_payload) => match video_payload {
                Ok(video_bytes) => {
                    if video_track.write(&video_bytes).await.is_err() {
                        error!("Error occurred while writing to video track for session");
                    }
                }
                Err(err) => {
                    error!("Error while reading video bytes: {err}");
                }
            },
            Payload::Audio(audio_payload) => match audio_payload {
                Ok(audio_bytes) => {
                    if audio_track.write(&audio_bytes).await.is_err() {
                        error!("Error occurred while writing to video track for session");
                    }
                }
                Err(err) => {
                    error!("Error while reading audio bytes: {err}");
                }
            },
        }
    }
    if let Err(err) = client.delete(whip_session_url).send().await {
        error!("Error while sending delete whip session request: {}", err);
    }
    event_emitter.emit(Event::OutputDone(output_id_clone));
    debug!("Closing WHIP sender thread.")
}
