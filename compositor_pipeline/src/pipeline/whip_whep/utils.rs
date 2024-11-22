use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use anyhow::Error;
use axum::http::HeaderValue;
use tokio::sync::mpsc::Sender;

use crate::{
    pipeline::{input::whip::depayloader::Depayloader, whip_whep::WhipWhepState, EncodedChunk},
    queue::PipelineEvent,
};
use compositor_render::InputId;
use tracing::{error, warn};
use webrtc::{rtp_transceiver::rtp_codec::RTPCodecType, track::track_remote::TrackRemote};

pub async fn initialize_start_time(
    state: Arc<WhipWhepState>,
    input_id: &InputId,
    track_kind: RTPCodecType,
) -> Result<(), String> {
    let mut input_connections = state
        .input_connections
        .lock()
        .map_err(|_| "Failed to lock input connections".to_string())?;
    let connection = input_connections
        .get_mut(input_id)
        .ok_or_else(|| "InputID not found".to_string())?;

    match track_kind {
        RTPCodecType::Video if connection.start_time_vid.is_none() => {
            connection.start_time_vid = Some(Instant::now())
        }
        RTPCodecType::Audio if connection.start_time_aud.is_none() => {
            connection.start_time_aud = Some(Instant::now())
        }
        _ => {}
    }
    Ok(())
}

pub fn get_start_time(
    state: Arc<WhipWhepState>,
    input_id: &InputId,
    track_kind: RTPCodecType,
) -> Option<Duration> {
    let input_connections = match state.input_connections.lock() {
        Ok(connections) => connections,
        Err(_) => return None,
    };

    match track_kind {
        RTPCodecType::Video => input_connections
            .get(input_id)
            .and_then(|conn| conn.start_time_vid)
            .map(|t| t.elapsed()),
        RTPCodecType::Audio => input_connections
            .get(input_id)
            .and_then(|conn| conn.start_time_aud)
            .map(|t| t.elapsed()),
        _ => None,
    }
}

pub async fn handle_track(
    track: Arc<TrackRemote>,
    state: Arc<WhipWhepState>,
    input_id: InputId,
    depayloader: Arc<Mutex<Depayloader>>,
    sender: Sender<PipelineEvent<EncodedChunk>>,
) {
    let mut first_pts_current_stream = None;
    let track_kind = track.kind();
    let state_clone = state.clone();
    let _ = initialize_start_time(state, &input_id, track_kind).await;
    let time = get_start_time(state_clone, &input_id, track_kind);

    //TODO send PipelineEvent::NewPeerConnection to reset queue and decoder(drop remaining frames from previous stream)

    let mut first_chunk_flag = true;

    while let Ok((rtp_packet, _)) = track.read_rtp().await {
        let Ok(chunks) = depayloader.lock().unwrap().depayload(rtp_packet) else {
            warn!("RTP depayloading error",);
            continue;
        };
        if let Some(first_chunk) = chunks.get(0) {
            if first_chunk_flag {
                first_chunk_flag = false;
                first_pts_current_stream = Some(first_chunk.pts);
            }
        }

        for mut chunk in chunks {
            chunk.pts = chunk.pts + time.unwrap_or(Duration::from_secs(0))
                - first_pts_current_stream.unwrap_or(Duration::from_secs(0));
            if let Err(e) = sender.send(PipelineEvent::Data(chunk)).await {
                error!("Failed to send audio RTP packet: {e}");
            }
        }
    }
}

pub fn authorize_token(
    expected_token: Option<String>,
    auth_header_value: Option<&HeaderValue>,
) -> Result<(), Error> {
    match (expected_token, auth_header_value) {
        (Some(bearer_token), Some(auth_str)) => {
            let auth_str = auth_str
                .to_str()
                .map_err(|_| Error::msg("Invalid UTF-8 in header"))?;

            if auth_str.starts_with("Bearer ") && auth_str[7..] == bearer_token {
                Ok(())
            } else {
                Err(Error::msg("Invalid or mismatched token provided"))
            }
        }
        _ => Err(Error::msg(
            "Expected token and authorization header required",
        )),
    }
}
