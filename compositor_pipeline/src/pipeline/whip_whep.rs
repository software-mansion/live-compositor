use crate::pipeline::input::whip::depayloader::Depayloader;
use anyhow::Error;
use axum::{
    routing::{delete, get, options, patch, post},
    Router,
};
use compositor_render::InputId;
use config::read_config;
use handlers::{
    handle_options, handle_whip, status, terminate_whip_session, whip_ice_candidates_handler,
};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex, Weak},
    time::Instant,
};
use tokio::sync::mpsc;
use tracing::{info, warn};
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

mod config;
mod handlers;
mod utils;

use tokio::task;

use crate::{queue::PipelineEvent, Pipeline};

use super::EncodedChunk;

pub(crate) const VIDEO_PAYLOAD_TYPE: u8 = 96;
pub(crate) const AUDIO_PAYLOAD_TYPE: u8 = 111;

#[tokio::main]
pub async fn run_whip_whep_server(pipeline: Weak<Mutex<Pipeline>>) {
    let config = read_config();
    let port = config.api_port;

    if !config.start_whip_whep {
        return;
    }

    let pipeline_ctx = match pipeline.upgrade() {
        Some(pipeline) => pipeline.lock().unwrap().ctx.clone(),
        None => {
            warn!("Pipeline stopped.");
            return;
        }
    };

    let state = pipeline_ctx.whip_whep_state.clone();

    let app = Router::new()
        .route("/status", get(status))
        .route("/whip/:id", post(handle_whip))
        .route("/whip/:id", options(handle_options))
        .route("/session/:id", patch(whip_ice_candidates_handler))
        .route("/session/:id", delete(terminate_whip_session))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], port)))
        .await
        .unwrap();

    let server_task = task::spawn(async {
        let _ = axum::serve(listener, app).await;
    });
    info!("started http server");
    if let Err(e) = server_task.await {
        eprintln!("WHIP/WHEP server task failed: {:?}", e);
    }
}

#[derive(Debug)]
pub struct InputConnectionUtils {
    pub video_sender: Option<mpsc::Sender<PipelineEvent<EncodedChunk>>>,
    pub audio_sender: Option<mpsc::Sender<PipelineEvent<EncodedChunk>>>,
    pub bearer_token: Option<String>,
    pub peer_connection: Option<Arc<RTCPeerConnection>>,
    pub start_time_vid: Option<Instant>,
    pub start_time_aud: Option<Instant>,
    pub depayloader: Arc<Mutex<Depayloader>>,
}

#[derive(Debug)]
pub struct WhipWhepState {
    pub input_connections: Arc<Mutex<HashMap<InputId, InputConnectionUtils>>>,
}

impl WhipWhepState {
    pub fn new() -> Arc<Self> {
        Arc::new(WhipWhepState {
            input_connections: Arc::from(Mutex::new(HashMap::new())),
        })
    }
}

pub async fn init_pc(
    add_video_track: bool,
    add_audio_track: bool,
) -> Result<Arc<RTCPeerConnection>, Error> {
    let mut m = MediaEngine::default();
    m.register_default_codecs()?;

    m.register_codec(
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

    m.register_codec(
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
    registry = register_default_interceptors(registry, &mut m)?;

    // Create the API object with the MediaEngine
    let api = APIBuilder::new()
        .with_media_engine(m)
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
    let peer_connection = Arc::new(api.new_peer_connection(config).await.unwrap());
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
