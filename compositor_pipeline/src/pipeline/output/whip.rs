use compositor_render::OutputId;
use crossbeam_channel::{Receiver, Sender};
use establish_peer_connection::connect;
use init_peer_connection::init_pc;
use packet_stream::PacketStream;
use payloader::{Payload, Payloader, PayloadingError};
use reqwest::{Method, StatusCode, Url};
use std::sync::{atomic::AtomicBool, Arc};
use tracing::{debug, error, span, Level};
use url::ParseError;
use webrtc::{
    peer_connection::RTCPeerConnection,
    track::track_local::{track_local_static_rtp::TrackLocalStaticRTP, TrackLocalWriter},
};

use crate::{
    error::OutputInitError,
    event::Event,
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
    pub bearer_token: Option<String>,
    pub video: Option<VideoCodec>,
    pub audio: Option<AudioCodec>,
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
    BodyParsingError(String, reqwest::Error),

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
}

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
        let endpoint_url = options.endpoint_url.clone();
        let bearer_token = options.bearer_token.clone();
        let output_id = output_id.clone();
        let should_close2 = should_close.clone();
        let event_emitter = pipeline_ctx.event_emitter.clone();
        let request_keyframe_sender = request_keyframe_sender.clone();
        let tokio_rt = pipeline_ctx.tokio_rt.clone();

        std::thread::Builder::new()
            .name(format!("WHIP sender for output {}", output_id))
            .spawn(move || {
                let _span = span!(
                    Level::INFO,
                    "WHIP sender",
                    output_id = output_id.to_string()
                )
                .entered();
                start_whip_sender_thread(
                    endpoint_url,
                    bearer_token,
                    should_close2,
                    packet_stream,
                    request_keyframe_sender,
                    tokio_rt,
                );
                event_emitter.emit(Event::OutputDone(output_id));
                debug!("Closing WHIP sender thread.")
            })
            .unwrap();

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

fn start_whip_sender_thread(
    endpoint_url: String,
    bearer_token: Option<String>,
    should_close: Arc<AtomicBool>,
    packet_stream: PacketStream,
    request_keyframe_sender: Option<Sender<()>>,
    tokio_rt: Arc<tokio::runtime::Runtime>,
) {
    tokio_rt.block_on(async {
        let client = reqwest::Client::new();
        let peer_connection: Arc<RTCPeerConnection>;
        let video_track: Arc<TrackLocalStaticRTP>;
        let audio_track: Arc<TrackLocalStaticRTP>;
        match init_pc().await {
            Ok((pc, video, audio)) => {
                peer_connection = pc;
                video_track = video;
                audio_track = audio;
            }
            Err(err) => {
                error!("Error occured while initializing peer connection: {err}");
                return;
            }
        }
        let whip_session_url = match connect(
            peer_connection,
            endpoint_url,
            bearer_token,
            should_close.clone(),
            tokio_rt.clone(),
            client.clone(),
            request_keyframe_sender,
        )
        .await
        {
            Ok(val) => val,
            Err(err) => {
                error!("{err}");
                return;
            }
        };

        for chunk in packet_stream {
            if should_close.load(std::sync::atomic::Ordering::Relaxed) {
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
        let _ = client.delete(whip_session_url).send().await;
    });
}
