use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, Response, StatusCode},
    response::IntoResponse,
};

use compositor_render::InputId;
use webrtc_util::{Marshal, MarshalSize};

use crate::pipeline::whip_whep::{init_pc, InputConnectionUtils, WhipWhepState};
use serde_json::{json, Value};
use tracing::{error, info};
use webrtc::{
    ice_transport::ice_candidate::RTCIceCandidateInit,
    peer_connection::{self, sdp::session_description::RTCSessionDescription},
    rtp_transceiver::rtp_codec::RTPCodecType,
};

pub async fn status() -> (StatusCode, axum::Json<Value>) {
    info!("[status] got request");
    (StatusCode::OK, axum::Json(json!({})))
}

pub async fn handle_whip(
    State(state): State<Arc<WhipWhepState>>,
    headers: HeaderMap,
    offer: String,
) -> Response<Body> {
    info!("[whip] got headers: {headers:?}");
    info!("[whip] got request: {offer}");

    //get id from bearer token
    // let bearer_token = todo!();
    let input_id = InputId(Arc::from("input_1"));

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

    let video_sender = state
        .input_connections
        .lock()
        .unwrap()
        .get(&input_id)
        .unwrap()
        .video_sender
        .clone()
        .unwrap();

    let audio_sender = state
        .input_connections
        .lock()
        .unwrap()
        .get(&input_id)
        .unwrap()
        .audio_sender
        .clone()
        .unwrap();

    info!(
        "video sender closed >>>>>>>> {:?}",
        video_sender.is_closed()
    );
    info!(
        "audio sender closed >>>>>>>> {:?}",
        audio_sender.is_closed()
    );

    let peer_connection = init_pc().await;
    state
        .input_connections
        .lock()
        .unwrap()
        .get_mut(&input_id)
        .unwrap()
        .peer_connection = Some(peer_connection.clone());
    state
        .input_connections
        .lock()
        .unwrap()
        .get_mut(&input_id)
        .unwrap()
        .bearer_token = Some(input_id.clone().to_string());

    info!(
        "=============== {:?}",
        state.input_connections.lock().unwrap().get(&input_id)
    );

    peer_connection.on_track(Box::new(move |track, _, _| {
        let audio_sender_clone = audio_sender.clone();
        let video_sender_clone = video_sender.clone();
        tokio::spawn(async move {
            let track_kind = track.kind();

            while let Ok((rtp_packet, _)) = track.read_rtp().await {
                // Send RTP packets through the channel
                if track_kind == RTPCodecType::Audio {
                    if let Err(e) = audio_sender_clone.send(rtp_packet).await {
                        error!("Failed to send audio RTP packet: {e}");
                    }
                } else if track_kind == RTPCodecType::Video {
                    if let Err(e) = video_sender_clone.send(rtp_packet).await {
                        error!("Failed to send video RTP packet: {e}");
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

    println!("{:?}", state);

    Response::builder()
        .status(StatusCode::CREATED)
        .header("Content-Type", "application/sdp")
        .header("Access-Control-Expose-Headers", "Location")
        .header("Location", "/session")
        .body(Body::from(sdp.sdp.to_string()))
        .unwrap()
}

pub async fn whip_ice_candidates_handler(
    State(state): State<Arc<WhipWhepState>>,
    headers: HeaderMap,
    candidate: String,
) -> (StatusCode, impl IntoResponse) {
    info!("[session] received candidate: {candidate:?}");
    info!("[session] received candidate: {headers:?}");

    //get id from bearer token
    let input_id = InputId(Arc::from("frst"));

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
        state
            .input_connections
            .lock()
            .unwrap()
            .get(&input_id)
            .unwrap()
            .video_sender
            .clone()
            .unwrap(),
    );

    drop(
        state
            .input_connections
            .lock()
            .unwrap()
            .get(&input_id)
            .unwrap()
            .audio_sender
            .clone()
            .unwrap(),
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
