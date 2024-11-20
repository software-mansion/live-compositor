use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::Error;
use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, Response, StatusCode},
    response::IntoResponse,
};

use crate::{
    pipeline::whip_whep::{init_pc, WhipWhepState},
    queue::PipelineEvent,
};
use compositor_render::InputId;
use serde_json::{json, Value};
use tracing::{error, info, warn};
use webrtc::{
    ice_transport::ice_candidate::RTCIceCandidateInit,
    peer_connection::sdp::session_description::RTCSessionDescription,
    rtp_transceiver::rtp_codec::RTPCodecType,
};

pub async fn status() -> (StatusCode, axum::Json<Value>) {
    info!("[status] got request");
    (StatusCode::OK, axum::Json(json!({})))
}

pub async fn handle_whip(
    Path(id): Path<String>,
    State(state): State<Arc<WhipWhepState>>,
    headers: HeaderMap,
    offer: String,
) -> Response<Body> {
    info!("[whip] got headers: {headers:?}");
    info!("[whip] got request: {offer}");

    let input_id = InputId(Arc::from(id.clone()));

    // Validate that the Content-Type is `application/sdp`
    if let Some(content_type) = headers.get("Content-Type") {
        if content_type.as_bytes() != b"application/sdp" {
            error!("Invalid Content-Type, expecting application/sdp");
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from(
                    "Invalid Content-Type, expecting application/sdp",
                ))
                .unwrap();
        }
    } else {
        error!("Missing Content-Type header");
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from("Missing Content-Type header"))
            .unwrap();
    }

    let video_sender;
    let audio_sender;
    let depayloader;
    let bearer_token;

    if let Ok(connections) = state.input_connections.lock() {
        if let Some(connection) = connections.get(&input_id) {
            video_sender = connection.video_sender.clone();
            audio_sender = connection.audio_sender.clone();
            depayloader = connection.depayloader.clone();
            bearer_token = connection.bearer_token.clone();
            if let Err(msg) = authorize_token(bearer_token, headers.get("Authorization")) {
                return Response::builder()
                    .status(StatusCode::NO_CONTENT)
                    .body(Body::from(msg.to_string()))
                    .unwrap();
            }
        } else {
            return Response::builder()
                .status(StatusCode::NO_CONTENT)
                .body(Body::from(""))
                .unwrap();
        }
    } else {
        return Response::builder()
            .status(StatusCode::NO_CONTENT)
            .body(Body::from(""))
            .unwrap();
    }

    let peer_connection = init_pc(video_sender.is_some(), audio_sender.is_some()).await;

    if let Ok(mut connections) = state.input_connections.lock() {
        if let Some(connection) = connections.get_mut(&input_id) {
            connection.peer_connection = Some(peer_connection.clone());
        } else {
            return Response::builder()
                .status(StatusCode::NO_CONTENT)
                .body(Body::from("Peer connection initialization error"))
                .unwrap();
        }
    }

    peer_connection.on_track(Box::new(move |track, _, _| {
        let video_sender_clone = video_sender.clone();
        let audio_sender_clone = audio_sender.clone();
        let state_clone = state.clone();
        let input_id_clone = input_id.clone();
        let depayloader_clone = depayloader.clone();

        tokio::spawn(async move {
            let track_kind = track.kind();
            let mut first_curr_pts_vid = None;
            let mut first_curr_pts_aud = None;
            let mut time_vid = None;
            let mut time_aud = None;

            //TODO send PipelineEvent::NewPeerConnection to reset queue and decoder(drop remaining frames from previous stream)

            if track_kind == RTPCodecType::Video {
                if let Some(start_time) = state_clone
                    .input_connections
                    .lock()
                    .unwrap()
                    .get(&input_id_clone)
                    .unwrap()
                    .start_time_vid
                {
                    info!("sent newpeer");
                    time_vid = Some(start_time.elapsed());
                }
                if state_clone
                    .input_connections
                    .lock()
                    .unwrap()
                    .get(&input_id_clone)
                    .unwrap()
                    .start_time_vid
                    .is_none()
                {
                    state_clone
                        .input_connections
                        .lock()
                        .unwrap()
                        .get_mut(&input_id_clone)
                        .unwrap()
                        .start_time_vid = Some(Instant::now())
                }
            } else if track_kind == RTPCodecType::Audio {
                if let Some(start_time) = state_clone
                    .input_connections
                    .lock()
                    .unwrap()
                    .get(&input_id_clone)
                    .unwrap()
                    .start_time_aud
                {
                    info!("sent newpeer");
                    time_aud = Some(start_time.elapsed());
                }
                if state_clone
                    .input_connections
                    .lock()
                    .unwrap()
                    .get(&input_id_clone)
                    .unwrap()
                    .start_time_aud
                    .is_none()
                {
                    state_clone
                        .input_connections
                        .lock()
                        .unwrap()
                        .get_mut(&input_id_clone)
                        .unwrap()
                        .start_time_aud = Some(Instant::now())
                }
            }
            let mut flag = true;

            while let Ok((rtp_packet, _)) = track.read_rtp().await {
                // Send RTP packets through the channel
                let Ok(chunks) = depayloader_clone.lock().unwrap().depayload(rtp_packet) else {
                    warn!("RTP depayloading error",);
                    continue;
                };
                if let Some(first_chunk) = chunks.get(0) {
                    if flag {
                        flag = false;
                        if track_kind == RTPCodecType::Video {
                            first_curr_pts_vid = Some(first_chunk.pts);
                        } else if track_kind == RTPCodecType::Audio {
                            first_curr_pts_aud = Some(first_chunk.pts);
                        }
                    }
                }

                for mut chunk in chunks {
                    if track_kind == RTPCodecType::Video {
                        info!(
                            "Video: {:?} {:?} {:?}",
                            chunk.pts, time_vid, first_curr_pts_vid
                        );

                        chunk.pts = chunk.pts + time_vid.unwrap_or(Duration::from_secs(0))
                            - first_curr_pts_vid.unwrap_or(Duration::from_secs(0));
                        if let Some(ref sender) = video_sender_clone {
                            if let Err(e) = sender.send(PipelineEvent::Data(chunk)).await {
                                error!("Failed to send audio RTP packet: {e}");
                            }
                        }
                    } else if track_kind == RTPCodecType::Audio {
                        info!(
                            "Audio: {:?} {:?} {:?}",
                            chunk.pts, time_vid, first_curr_pts_vid
                        );
                        chunk.pts = chunk.pts + time_aud.unwrap_or(Duration::from_secs(0))
                            - first_curr_pts_aud.unwrap_or(Duration::from_secs(0));

                        if let Some(ref sender) = audio_sender_clone {
                            if let Err(e) = sender.send(PipelineEvent::Data(chunk)).await {
                                error!("Failed to send audio RTP packet: {e}");
                            }
                        }
                    }
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
    peer_connection
        .set_remote_description(RTCSessionDescription::offer(offer).unwrap())
        .await
        .unwrap();

    // Create and set the local SDP answer
    let answer = peer_connection.create_answer(None).await.unwrap();

    let mut gather = peer_connection.gathering_complete_promise().await;

    peer_connection.set_local_description(answer).await.unwrap();

    let _ = gather.recv().await;

    let sdp = peer_connection.local_description().await.unwrap();
    info!("Sending SDP answer: {sdp:?}");

    Response::builder()
        .status(StatusCode::CREATED)
        .header("Content-Type", "application/sdp")
        .header("Access-Control-Expose-Headers", "Location")
        .header("Location", "/session/".to_string() + &id)
        .body(Body::from(sdp.sdp.to_string()))
        .unwrap()
}

pub async fn whip_ice_candidates_handler(
    Path(id): Path<String>,
    State(state): State<Arc<WhipWhepState>>,
    headers: HeaderMap,
    candidate: String,
) -> (StatusCode, impl IntoResponse) {
    info!("[session] received candidate: {candidate:?}");
    info!("[session] received candidate: {headers:?}");

    let input_id = InputId(Arc::from(id));

    let candidate: Value = serde_json::from_str(&candidate).unwrap();

    let candidate_str = candidate["candidate"].as_str().unwrap_or("");
    let candidate_obj = RTCIceCandidateInit {
        candidate: candidate_str.to_string(),
        sdp_mid: candidate["sdpMid"].as_str().map(|s| s.to_string()),
        sdp_mline_index: candidate["sdpMLineIndex"].as_u64().map(|i| i as u16),
        ..Default::default()
    };

    let pc = state.input_connections.lock().unwrap()[&input_id]
        .peer_connection
        .clone()
        .unwrap();
    {
        let _ = pc.add_ice_candidate(candidate_obj).await;
    }

    (
        StatusCode::NO_CONTENT,
        axum::Json(json!({"status": "Candidate added"})),
    )
}

pub async fn handle_options() -> Response<Body> {
    // TODO
    Response::builder()
        .status(StatusCode::OK)
        .header("Accept-Post", "application/sdp")
        .body(Body::empty())
        .unwrap()
}

pub async fn terminate_whip_session(
    State(state): State<Arc<WhipWhepState>>,
    _headers: HeaderMap,
) -> (StatusCode, impl IntoResponse) {
    info!("[whip] terminating session");

    //get id from bearer token
    let input_id = InputId(Arc::from("input_1"));

    let pc = match state.input_connections.lock().unwrap()[&input_id]
        .peer_connection
        .clone()
    {
        Some(pc) => pc,
        None => {
            error!("Peer connection already terminated");
            return (
                StatusCode::NO_CONTENT,
                axum::Json(json!({"status": "Session terminated"})),
            );
        }
    };

    drop(
        state.input_connections.lock().unwrap()[&input_id]
            .video_sender
            .clone(),
    );

    if let Err(err) = pc.close().await {
        error!("Failed to close peer connection: {:?}", err);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(json!({"status": "Failed to terminate session"})),
        );
    }
    info!("[whip] session terminated");
    (
        StatusCode::NO_CONTENT,
        axum::Json(json!({"status": "Session terminated"})),
    )
}

fn authorize_token(
    expected_token: Option<String>,
    auth_header_value: Option<&HeaderValue>,
) -> Result<(), Error> {
    if let (Some(bearer_token), Some(auth_str)) = (expected_token, auth_header_value) {
        if auth_str.as_bytes()[7..] != *bearer_token.as_bytes() {
            return Err(Error::msg("Bad token provided"));
        }
        return Ok(());
    } else {
        return Err(Error::msg("Bad token provided"));
    }
}
