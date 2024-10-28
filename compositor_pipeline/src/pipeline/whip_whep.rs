use axum::{
    routing::{delete, get, options, patch, post},
    Router,
};
use config::read_config;
use handlers::{
    handle_options, handle_whip, status, terminate_whip_session, whip_ice_candidates_handler,
};
use logger::init_logger;
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::sync::{mpsc, Notify};
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
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
mod logger;

use tokio::task;

#[derive(Clone)]
pub struct WhipUtils {
    pub peer_connection: Arc<RTCPeerConnection>,
}

#[tokio::main]
pub async fn start_whip_whep_server() {
    let config = read_config();
    init_logger(config.logger.clone());
    let port = config.api_port;
    
    if !config.start_whip_whep {
        return;
    }

    let state = init().await;

    let app = Router::new()
        .route("/status", get(status))
        // .route("/whep", post(handle_whep))
        .route("/whip", post(handle_whip))
        .route("/whip", options(handle_options))
        .route("/session", patch(whip_ice_candidates_handler))
        .route("/session", delete(terminate_whip_session))
        // .route("/resource/:id", patch(whep_ice_candidates_handler))
        // .route("/resource/:id", delete(terminate_whep_session))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], port)))
        .await
        .unwrap();

    let server_task = task::spawn(async {
        axum::serve(listener, app).await.unwrap();
    });
    info!("started http server");
    if let Err(e) = server_task.await {
        eprintln!("WHIP/WHEP server task failed: {:?}", e);
    }
}

pub struct WhipWhepState {
    pub whip: Arc<WhipUtils>,
    pub notifier: Arc<Notify>,
    pub video_receiver: Mutex<mpsc::Receiver<Vec<u8>>>,
    pub audio_receiver: Mutex<mpsc::Receiver<Vec<u8>>>,
}

pub async fn init() -> Arc<WhipWhepState> {
    let (_video_sender, video_receiver) = mpsc::channel(32);
    let (_audio_sender, audio_receiver) = mpsc::channel(32);

    Arc::new(WhipWhepState {
        whip: init_pc().await,
        notifier: Arc::new(Notify::new()),
        video_receiver: Mutex::new(video_receiver),
        audio_receiver: Mutex::new(audio_receiver),
    })
}

pub async fn init_pc() -> Arc<WhipUtils> {
    let mut m = MediaEngine::default();
    m.register_default_codecs().unwrap();

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
    )
    .unwrap();

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
    )
    .unwrap();

    // Create a InterceptorRegistry. This is the user configurable RTP/RTCP Pipeline.
    // This provides NACKs, RTCP Reports and other features. If you use `webrtc.NewPeerConnection`
    // this is enabled by default. If you are manually managing You MUST create a InterceptorRegistry
    // for each PeerConnection.
    let mut registry = Registry::new();

    // Use the default set of Interceptors
    registry = register_default_interceptors(registry, &mut m).unwrap();

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

    peer_connection
        .add_transceiver_from_kind(
            RTPCodecType::Audio,
            Some(RTCRtpTransceiverInit {
                direction: RTCRtpTransceiverDirection::Recvonly,
                send_encodings: vec![],
            }),
        )
        .await
        .unwrap();
    peer_connection
        .add_transceiver_from_kind(
            RTPCodecType::Video,
            Some(RTCRtpTransceiverInit {
                direction: RTCRtpTransceiverDirection::Recvonly,
                send_encodings: vec![],
            }),
        )
        .await
        .unwrap();

    Arc::new(WhipUtils { peer_connection })
}
