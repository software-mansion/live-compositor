use crate::pipeline::input::whip::depayloader::Depayloader;
use axum::{
    routing::{delete, get, patch, post},
    Router,
};
use compositor_render::InputId;
use error::WhipServerError;
use reqwest::StatusCode;
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::{
    signal,
    sync::{mpsc, oneshot},
};
use tower_http::cors::CorsLayer;
use tracing::{error, warn};
use webrtc::{
    peer_connection::{peer_connection_state::RTCPeerConnectionState, RTCPeerConnection},
    rtp_transceiver::rtp_codec::RTPCodecType,
};
use whip_handlers::{
    create_whip_session::handle_create_whip_session,
    new_whip_ice_candidates::handle_new_whip_ice_candidates,
    terminate_whip_session::handle_terminate_whip_session,
};

mod error;
mod init_peer_connection;
mod validate_bearer_token;
mod whip_handlers;

use crate::queue::PipelineEvent;

use super::EncodedChunk;

pub async fn run_whip_whep_server(
    port: u16,
    state: Arc<WhipWhepState>,
    shutdown_signal_receiver: oneshot::Receiver<()>,
) {
    let app = Router::new()
        .route("/status", get(status))
        .route("/whip/:id", post(handle_create_whip_session))
        .route("/session/:id", patch(handle_new_whip_ice_candidates))
        .route("/session/:id", delete(handle_terminate_whip_session))
        .layer(CorsLayer::permissive())
        .with_state(state.clone());

    let Ok(listener) = tokio::net::TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], port))).await
    else {
        warn!("TCP listener error");
        return;
    };

    if let Err(err) = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(shutdown_signal_receiver))
        .await
    {
        error!("Cannot serve WHIP/WHEP server task: {err:?}");
    };
    state.clear_state();
}

async fn shutdown_signal(receiver: oneshot::Receiver<()>) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    let terminate = async {
        if let Err(err) = receiver.await {
            warn!(
                "Error while receiving whip_whep server shutdown signal {:?}",
                err
            );
        }
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

pub async fn status() -> Result<(StatusCode, axum::Json<Value>), WhipServerError> {
    Ok((StatusCode::OK, axum::Json(json!({}))))
}

#[derive(Debug, Clone)]
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
    pub fn get_or_initialize_elapsed_start_time(
        &mut self,
        track_kind: RTPCodecType,
    ) -> Option<Duration> {
        match track_kind {
            RTPCodecType::Video => {
                let start_time = self.start_time_vid.get_or_insert_with(Instant::now);
                Some(start_time.elapsed())
            }
            RTPCodecType::Audio => {
                let start_time = self.start_time_aud.get_or_insert_with(Instant::now);
                Some(start_time.elapsed())
            }
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

    pub fn get_input_connection_options(
        &self,
        input_id: InputId,
    ) -> Result<WhipInputConnectionOptions, WhipServerError> {
        let connections = self.input_connections.lock().unwrap();
        if let Some(connection) = connections.get(&input_id) {
            if let Some(peer_connection) = connection.peer_connection.clone() {
                warn!("There is another stream streaming for given input {input_id:?}");
                if peer_connection.connection_state() == RTCPeerConnectionState::Connected {
                    return Err(WhipServerError::InternalError(format!(
                        "There is another stream streaming for given input {input_id:?}"
                    )));
                }
            }
            Ok(connection.clone())
        } else {
            Err(WhipServerError::NotFound(format!(
                "InputID {input_id:?} not found"
            )))
        }
    }

    pub async fn update_peer_connection(
        &self,
        input_id: InputId,
        peer_connection: Arc<RTCPeerConnection>,
    ) -> Result<(), WhipServerError> {
        let mut connections = self.input_connections.lock().unwrap();
        if let Some(connection) = connections.get_mut(&input_id) {
            connection.peer_connection = Some(peer_connection);
            Ok(())
        } else {
            Err(WhipServerError::InternalError(
                "Peer connection initialization error".to_string(),
            ))
        }
    }

    pub fn get_time_elapsed_from_input_start(
        &self,
        input_id: InputId,
        track_kind: RTPCodecType,
    ) -> Option<Duration> {
        let mut connections = self.input_connections.lock().unwrap();
        match connections.get_mut(&input_id) {
            Some(connection) => connection.get_or_initialize_elapsed_start_time(track_kind),
            None => {
                error!("InputID {input_id:?} not found");
                None
            }
        }
    }

    pub fn clear_state(&self) {
        let mut connections = self.input_connections.lock().unwrap();
        connections.clear();
    }
}
