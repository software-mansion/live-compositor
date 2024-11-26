use std::sync::{Arc, Mutex};

use crate::{
    pipeline::{
        input::whip::{depayloader::Depayloader, handle_track},
        whip_whep::{init_peer_connection, validate_token::validate_token, WhipWhepState},
        EncodedChunk,
    },
    queue::PipelineEvent,
};
use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, Response, StatusCode},
    response::IntoResponse,
};
use compositor_render::InputId;
use serde_json::{json, Value};
use tokio::sync::mpsc::Sender;
use tracing::{debug, error, info};
use webrtc::{
    ice_transport::ice_candidate::RTCIceCandidateInit,
    peer_connection::{sdp::session_description::RTCSessionDescription, RTCPeerConnection},
    rtp_transceiver::rtp_codec::RTPCodecType,
};

use super::WhipServerError;

pub async fn status() -> Result<(StatusCode, axum::Json<Value>), WhipServerError> {
    Ok((StatusCode::OK, axum::Json(json!({}))))
}

#[axum::debug_handler]
pub async fn handle_whip(
    Path(id): Path<String>,
    State(state): State<Arc<WhipWhepState>>,
    headers: HeaderMap,
    offer: String,
) -> Result<Response<Body>, WhipServerError> {
    let input_id = InputId(Arc::from(id.clone()));

    // Validate that the Content-Type is `application/sdp`
    if let Some(content_type) = headers.get("Content-Type") {
        if content_type.as_bytes() != b"application/sdp" {
            error!("Invalid Content-Type, expecting application/sdp");
            return Err(WhipServerError::InternalError(
                "Invalid Content-Type, expecting application/sdp".to_string(),
            ));
        }
    } else {
        error!("Missing Content-Type header");
        return Err(WhipServerError::BadRequest(
            "Missing Content-Type header".to_string(),
        ));
    }

    let video_sender: Option<Sender<PipelineEvent<EncodedChunk>>>;
    let audio_sender: Option<Sender<PipelineEvent<EncodedChunk>>>;
    let depayloader: Arc<Mutex<Depayloader>>;
    let bearer_token: Option<String>;
    let state_clone = state.clone();

    if let Ok(connections) = state_clone.input_connections.lock() {
        if let Some(connection) = connections.get(&input_id) {
            video_sender = connection.video_sender.clone();
            audio_sender = connection.audio_sender.clone();
            depayloader = connection.depayloader.clone();
            bearer_token = connection.bearer_token.clone();
        } else {
            return Err(WhipServerError::NotFound(format!(
                "InputID {input_id:?} not found"
            )));
        }
    } else {
        return Err(WhipServerError::InternalError(
            "Input connections lock error".to_string(),
        ));
    }
    validate_token(bearer_token, headers.get("Authorization")).await?;

    let peer_connection =
        init_peer_connection(video_sender.is_some(), audio_sender.is_some()).await?;

    if let Ok(mut connections) = state.input_connections.lock() {
        if let Some(connection) = connections.get_mut(&input_id) {
            connection.peer_connection = Some(peer_connection.clone());
        } else {
            return Err(WhipServerError::InternalError(
                "Peer connection initialization error".to_string(),
            ));
        }
    } else {
        return Err(WhipServerError::InternalError(
            "Cannot lock input connections".to_string(),
        ));
    }

    peer_connection.on_track(Box::new(move |track, _, _| {
        let track_kind = track.kind();
        let video_sender_clone = video_sender.clone();
        let audio_sender_clone = audio_sender.clone();
        let state_clone = state.clone();
        let input_id_clone = input_id.clone();
        let depayloader_clone = depayloader.clone();

        tokio::spawn(async move {
            match track_kind {
                RTPCodecType::Video => {
                    if let Some(sender) = video_sender_clone {
                        handle_track(
                            track,
                            state_clone,
                            input_id_clone,
                            depayloader_clone,
                            sender,
                        )
                        .await;
                    }
                }
                RTPCodecType::Audio => {
                    if let Some(sender) = audio_sender_clone {
                        handle_track(
                            track,
                            state_clone,
                            input_id_clone,
                            depayloader_clone,
                            sender,
                        )
                        .await;
                    }
                }
                _ => {
                    debug!("RTPCodecType not supported!")
                }
            }
        });
        Box::pin(async {})
    }));

    // Handle ICE connection state changes (logging for debugging)
    peer_connection.on_ice_connection_state_change(Box::new(move |state| {
        info!("ICE connection state changed: {state:?}");
        Box::pin(async {})
    }));

    // Set the remote SDP offer
    let description = RTCSessionDescription::offer(offer)?;

    peer_connection.set_remote_description(description).await?;
    let answer = peer_connection.create_answer(None).await?;

    let mut gather = peer_connection.gathering_complete_promise().await;

    peer_connection.set_local_description(answer).await?;

    let _ = gather.recv().await;

    let Some(sdp) = peer_connection.local_description().await else {
        return Err(WhipServerError::InternalError(
            "Set local description error".to_string(),
        ));
    };
    debug!("Sending SDP answer: {sdp:?}");

    let body = Body::from(sdp.sdp.to_string());
    let response = Response::builder()
        .status(StatusCode::CREATED)
        .header("Content-Type", "application/sdp")
        .header("Access-Control-Expose-Headers", "Location")
        .header("Location", format!("/session/{}", id))
        .body(body)?;
    Ok(response)
}

#[axum::debug_handler]
pub async fn whip_ice_candidates_handler(
    Path(id): Path<String>,
    State(state): State<Arc<WhipWhepState>>,
    candidate: String,
) -> Result<(StatusCode, impl IntoResponse), WhipServerError> {
    let input_id = InputId(Arc::from(id));

    let candidate: Value = serde_json::from_str(&candidate)?;

    let candidate_str = candidate["candidate"].as_str().unwrap_or("");
    let candidate_obj = RTCIceCandidateInit {
        candidate: candidate_str.to_string(),
        sdp_mid: candidate["sdpMid"].as_str().map(|s| s.to_string()),
        sdp_mline_index: candidate["sdpMLineIndex"].as_u64().map(|i| i as u16),
        ..Default::default()
    };

    let peer_connection: Option<Arc<RTCPeerConnection>>;

    if let Ok(connections) = state.input_connections.lock() {
        if let Some(connection_opts) = connections.get(&input_id) {
            peer_connection = connection_opts.peer_connection.clone()
        } else {
            return Err(WhipServerError::NotFound(format!(
                "InputID {input_id:?} not found"
            )));
        }
    } else {
        return Err(WhipServerError::InternalError(
            "Input connections lock error".to_string(),
        ));
    }

    if let Some(pc) = peer_connection {
        pc.add_ice_candidate(candidate_obj).await?;
    } else {
        return Err(WhipServerError::InternalError(format!(
            "None peer connection for {input_id:?}"
        )));
    }

    Ok((
        StatusCode::NO_CONTENT,
        axum::Json(json!({"status": "Candidate added"})),
    ))
}

pub async fn handle_options() -> Result<Response<Body>, WhipServerError> {
    // TODO
    Ok(Response::builder()
        .status(StatusCode::NOT_IMPLEMENTED)
        .header("Accept-Post", "application/sdp")
        .body(Body::empty())?)
}

pub async fn terminate_whip_session(
    Path(id): Path<String>,
    State(state): State<Arc<WhipWhepState>>,
) -> Result<(StatusCode, impl IntoResponse), WhipServerError> {
    let input_id = InputId(Arc::from(id));
    let peer_connection;

    if let Ok(connections) = state.input_connections.lock() {
        if let Some(connection) = connections.get(&input_id) {
            peer_connection = connection.peer_connection.clone();
            drop(connection.audio_sender.clone());
            drop(connection.video_sender.clone());
        } else {
            return Err(WhipServerError::NotFound(format!(
                "InputID {input_id:?} not found"
            )));
        }
    } else {
        return Err(WhipServerError::InternalError(
            "Input connections lock error".to_string(),
        ));
    }

    if let Some(peer_connection) = peer_connection {
        peer_connection.close().await?;
    } else {
        return Err(WhipServerError::InternalError(format!(
            "None peer connection for {input_id:?}"
        )));
    }

    info!("[whip] session terminated for input: {:?}", input_id);
    Ok((
        StatusCode::NO_CONTENT,
        axum::Json(json!({"status": "Session terminated"})),
    ))
}
