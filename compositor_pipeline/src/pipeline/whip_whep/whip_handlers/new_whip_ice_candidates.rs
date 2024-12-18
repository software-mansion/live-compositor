use crate::pipeline::whip_whep::{
    error::WhipServerError, validate_bearer_token::validate_token, WhipWhepState,
};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};
use compositor_render::InputId;

use std::sync::Arc;
use webrtc::{
    ice_transport::ice_candidate::RTCIceCandidateInit, peer_connection::RTCPeerConnection,
};

pub async fn handle_new_whip_ice_candidates(
    Path(id): Path<String>,
    State(state): State<Arc<WhipWhepState>>,
    headers: HeaderMap,
    sdp_fragment_content: String,
) -> Result<StatusCode, WhipServerError> {
    let content_type = headers
        .get("Content-Type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");

    if content_type != "application/trickle-ice-sdpfrag" {
        return Err(WhipServerError::BadRequest(
            "Invalid content type".to_owned(),
        ));
    }
    let bearer_token: Option<String>;
    let input_id = InputId(Arc::from(id));

    if let Ok(connections) = state.input_connections.lock() {
        if let Some(connection) = connections.get(&input_id) {
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

    if let Some(peer_connection) = peer_connection {
        for candidate in ice_fragment_unmarshal(&sdp_fragment_content) {
            if let Err(err) = peer_connection.add_ice_candidate(candidate.clone()).await {
                return Err(WhipServerError::BadRequest(format!(
                    "Cannot add ice_candidate {candidate:?} for input {input_id:?}: {err:?}"
                )));
            }
        }
    } else {
        return Err(WhipServerError::InternalError(format!(
            "None peer connection for {input_id:?}"
        )));
    }

    Ok(StatusCode::NO_CONTENT)
}

pub fn ice_fragment_unmarshal(sdp_content: &str) -> Vec<RTCIceCandidateInit> {
    let lines = sdp_content.split("\n");
    let mut candidates = Vec::new();
    let mut mid = None;
    let mut mid_num = None;

    for line in lines {
        if line.starts_with("a=mid:") {
            mid = line
                .split_once(':')
                .map(|(_, value)| value.trim().to_string());
            mid_num = mid.as_ref().and_then(|m| m.parse::<u16>().ok());
        }
        if line.starts_with("a=candidate:") {
            candidates.push(RTCIceCandidateInit {
                candidate: line[2..].to_string(),
                sdp_mid: mid.clone(),
                sdp_mline_index: mid_num,
                ..Default::default()
            });
        }
    }
    candidates
}
