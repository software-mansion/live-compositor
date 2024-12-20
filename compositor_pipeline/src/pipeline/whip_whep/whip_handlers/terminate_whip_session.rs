use crate::pipeline::whip_whep::{
    error::WhipServerError, validate_bearer_token::validate_token, WhipWhepState,
};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};
use compositor_render::InputId;
use std::sync::Arc;
use tracing::info;

pub async fn handle_terminate_whip_session(
    Path(id): Path<String>,
    State(state): State<Arc<WhipWhepState>>,
    headers: HeaderMap,
) -> Result<StatusCode, WhipServerError> {
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
    let peer_connection;

    if let Ok(mut connections) = state.input_connections.lock() {
        if let Some(connection) = connections.get_mut(&input_id) {
            peer_connection = connection.peer_connection.clone();
            connection.peer_connection = None;
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
    Ok(StatusCode::OK)
}
