use crate::pipeline::input::whip::depayloader::Depayloader;
use axum::{
    routing::{delete, get, patch, post},
    Router,
};
use compositor_render::InputId;
use error::WhipServerError;
use handlers::{handle_whip, status, terminate_whip_session, whip_ice_candidates_handler};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex, Weak},
    time::{Duration, Instant},
};
use tokio::sync::mpsc;
use tower_http::cors::CorsLayer;
use tracing::{error, warn};
use webrtc::{
    api::{
        interceptor_registry::register_default_interceptors,
        media_engine::{MediaEngine, MIME_TYPE_H264, MIME_TYPE_OPUS},
        APIBuilder,
    },
    ice_transport::ice_server::RTCIceServer,
    interceptor::registry::Registry,
    peer_connection::{configuration::RTCConfiguration, RTCPeerConnection},
    rtp_transceiver::{
        rtp_codec::{RTCRtpCodecCapability, RTCRtpCodecParameters, RTPCodecType},
        rtp_transceiver_direction::RTCRtpTransceiverDirection,
        RTCRtpTransceiverInit,
    },
};

mod error;
mod handlers;
mod helpers;

use crate::{queue::PipelineEvent, Pipeline};

use super::EncodedChunk;

pub(crate) const VIDEO_PAYLOAD_TYPE: u8 = 96;
pub(crate) const OPUS_PAYLOAD_TYPE: u8 = 111;

pub async fn run_whip_whep_server(pipeline: Weak<Mutex<Pipeline>>) {
    let pipeline_ctx = match pipeline.upgrade() {
        Some(pipeline) => pipeline.lock().unwrap().ctx.clone(),
        None => {
            warn!("Pipeline stopped.");
            return;
        }
    };

    if !pipeline_ctx.start_whip_whep {
        return;
    }

    let state = pipeline_ctx.whip_whep_state;
    let port = pipeline_ctx.whip_whep_server_port;

    let app = Router::new()
        .route("/status", get(status))
        .route("/whip/:id", post(handle_whip))
        .route("/session/:id", patch(whip_ice_candidates_handler))
        .route("/session/:id", delete(terminate_whip_session))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let Ok(listener) = tokio::net::TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], port))).await
    else {
        warn!("TCP listener error");
        return;
    };

    if let Err(err) = axum::serve(listener, app).await {
        error!("Cannot serve WHIP/WHEP server task: {err:?}");
    };
}

#[derive(Debug)]
pub struct WhipInputConnectionOptions {
    pub video_sender: Option<mpsc::Sender<PipelineEvent<EncodedChunk>>>,
    pub audio_sender: Option<mpsc::Sender<PipelineEvent<EncodedChunk>>>,
    pub bearer_token: Option<String>,
    pub peer_connection: Option<Arc<RTCPeerConnection>>,
    pub start_time_vid: Option<Instant>,
    pub start_time_aud: Option<Instant>,
    pub depayloader: Arc<Mutex<Depayloader>>,
}

impl WhipInputConnectionOptions {
    pub fn set_start_time(&mut self, track_kind: RTPCodecType) -> Result<(), String> {
        match track_kind {
            RTPCodecType::Video if self.start_time_vid.is_none() => {
                self.start_time_vid = Some(Instant::now())
            }
            RTPCodecType::Audio if self.start_time_aud.is_none() => {
                self.start_time_aud = Some(Instant::now())
            }
            _ => {}
        }
        Ok(())
    }

    pub fn get_start_time(&self, track_kind: RTPCodecType) -> Option<Duration> {
        match track_kind {
            RTPCodecType::Video => self.start_time_vid.map(|t| t.elapsed()),
            RTPCodecType::Audio => self.start_time_aud.map(|t| t.elapsed()),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct WhipWhepState {
    pub input_connections: Arc<Mutex<HashMap<InputId, WhipInputConnectionOptions>>>,
}

impl WhipWhepState {
    pub fn new() -> Arc<Self> {
        Arc::new(WhipWhepState {
            input_connections: Arc::from(Mutex::new(HashMap::new())),
        })
    }
}

pub async fn init_peer_connection(
    add_video_track: bool,
    add_audio_track: bool,
) -> Result<Arc<RTCPeerConnection>, WhipServerError> {
    let mut media_engine = MediaEngine::default();
    media_engine.register_default_codecs()?;

    media_engine.register_codec(
        RTCRtpCodecParameters {
            capability: RTCRtpCodecCapability {
                mime_type: MIME_TYPE_H264.to_owned(),
                clock_rate: 90000,
                channels: 0,
                sdp_fmtp_line: "".to_owned(),
                rtcp_feedback: vec![],
            },
            payload_type: 96,
            ..Default::default()
        },
        RTPCodecType::Video,
    )?;

    media_engine.register_codec(
        RTCRtpCodecParameters {
            capability: RTCRtpCodecCapability {
                mime_type: MIME_TYPE_OPUS.to_owned(),
                clock_rate: 48000,
                channels: 2,
                sdp_fmtp_line: "".to_owned(),
                rtcp_feedback: vec![],
            },
            payload_type: 97,
            ..Default::default()
        },
        RTPCodecType::Audio,
    )?;

    // Create a InterceptorRegistry. This is the user configurable RTP/RTCP Pipeline.
    // This provides NACKs, RTCP Reports and other features. If you use `webrtc.NewPeerConnection`
    // this is enabled by default. If you are manually managing You MUST create a InterceptorRegistry
    // for each PeerConnection.
    let mut registry = Registry::new();

    // Use the default set of Interceptors
    registry = register_default_interceptors(registry, &mut media_engine)?;

    // Create the API object with the MediaEngine
    let api = APIBuilder::new()
        .with_media_engine(media_engine)
        .with_interceptor_registry(registry)
        .build();

    // Prepare the configuration
    let config = RTCConfiguration {
        ice_servers: vec![RTCIceServer {
            urls: vec!["stun:stun.l.google.com:19302".to_owned()],
            ..Default::default()
        }],
        ..Default::default()
    };

    // Create a new RTCPeerConnection
    let peer_connection = Arc::new(api.new_peer_connection(config).await?);
    if add_video_track {
        peer_connection
            .add_transceiver_from_kind(
                RTPCodecType::Audio,
                Some(RTCRtpTransceiverInit {
                    direction: RTCRtpTransceiverDirection::Recvonly,
                    send_encodings: vec![],
                }),
            )
            .await?;
    }
    if add_audio_track {
        peer_connection
            .add_transceiver_from_kind(
                RTPCodecType::Video,
                Some(RTCRtpTransceiverInit {
                    direction: RTCRtpTransceiverDirection::Recvonly,
                    send_encodings: vec![],
                }),
            )
            .await?;
    }

    Ok(peer_connection)
}
