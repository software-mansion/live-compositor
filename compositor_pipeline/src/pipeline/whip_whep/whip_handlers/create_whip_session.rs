use crate::pipeline::{
    input::whip::process_track_stream,
    whip_whep::{
        bearer_token::validate_token, error::WhipServerError, init_peer_connection, WhipWhepState,
    },
};
use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, Response, StatusCode},
};
use compositor_render::InputId;
use init_peer_connection::init_peer_connection;
use std::{sync::Arc, time::Duration};
use tokio::{sync::watch, time::timeout};
use tracing::{debug, info};
use urlencoding::encode;
use webrtc::{
    ice_transport::ice_gatherer_state::RTCIceGathererState,
    peer_connection::{sdp::session_description::RTCSessionDescription, RTCPeerConnection},
    rtp_transceiver::rtp_codec::RTPCodecType,
};

pub async fn handle_create_whip_session(
    Path(id): Path<String>,
    State(state): State<Arc<WhipWhepState>>,
    headers: HeaderMap,
    offer: String,
) -> Result<Response<Body>, WhipServerError> {
    let input_id = InputId(Arc::from(id.clone()));

    validate_sdp_content_type(&headers)?;

    let input_components = state.get_input_connection_options(input_id.clone())?;

    validate_token(input_components.bearer_token, headers.get("Authorization")).await?;

    //Deleting previous peer_connection on this input which was not in Connected state
    if let Some(connection) = input_components.peer_connection {
        if let Err(err) = connection.close().await {
            return Err(WhipServerError::InternalError(format!(
                "Cannot close previously existing peer connection {input_id:?}: {err:?}"
            )));
        }
    }

    let peer_connection = init_peer_connection(
        input_components.video_sender.is_some(),
        input_components.audio_sender.is_some(),
        state.stun_servers.to_vec(),
    )
    .await?;

    state
        .update_peer_connection(input_id.clone(), peer_connection.clone())
        .await?;

    peer_connection.on_track(Box::new(move |track, _, _| {
        let track_kind = track.kind();
        let state_clone = state.clone();
        let input_id_clone = input_id.clone();
        let depayloader_clone = input_components.depayloader.clone();
        let sender = match track_kind {
            RTPCodecType::Video => input_components.video_sender.clone(),
            RTPCodecType::Audio => input_components.audio_sender.clone(),
            _ => {
                debug!("RTPCodecType not supported!");
                None
            }
        };

        //tokio::spawn is necessary to concurrently process audio and video track
        Box::pin(async {
            if let Some(sender) = sender {
                tokio::spawn(async move {
                    process_track_stream(
                        track,
                        state_clone,
                        input_id_clone,
                        depayloader_clone,
                        sender,
                    )
                    .await;
                });
            }
        })
    }));

    peer_connection.on_ice_connection_state_change(Box::new(move |state| {
        info!("ICE connection state changed: {state:?}");
        Box::pin(async {})
    }));

    let description = RTCSessionDescription::offer(offer)?;

    peer_connection.set_remote_description(description).await?;
    let answer = peer_connection.create_answer(None).await?;
    peer_connection.set_local_description(answer).await?;
    gather_ice_candidates_for_one_second(peer_connection.clone()).await;

    let Some(sdp) = peer_connection.local_description().await else {
        return Err(WhipServerError::InternalError(
            "Local description is not set, cannot read it".to_string(),
        ));
    };
    debug!("Sending SDP answer: {sdp:?}");

    let body = Body::from(sdp.sdp.to_string());
    let response = Response::builder()
        .status(StatusCode::CREATED)
        .header("Content-Type", "application/sdp")
        .header("Access-Control-Expose-Headers", "Location")
        .header("Location", format!("/session/{}", encode(&id)))
        .body(body)?;
    Ok(response)
}

pub fn validate_sdp_content_type(headers: &HeaderMap) -> Result<(), WhipServerError> {
    if let Some(content_type) = headers.get("Content-Type") {
        if content_type.as_bytes() != b"application/sdp" {
            return Err(WhipServerError::InternalError(
                "Invalid Content-Type".to_string(),
            ));
        }
    } else {
        return Err(WhipServerError::BadRequest(
            "Missing Content-Type header".to_string(),
        ));
    }
    Ok(())
}

pub async fn gather_ice_candidates_for_one_second(peer_connection: Arc<RTCPeerConnection>) {
    let (sender, mut receiver) = watch::channel(RTCIceGathererState::Unspecified);

    peer_connection.on_ice_gathering_state_change(Box::new(move |gatherer_state| {
        if let Err(err) = sender.send(gatherer_state) {
            debug!("Cannot send gathering state: {err:?}");
        };
        Box::pin(async {})
    }));

    let gather_candidates = async {
        while receiver.changed().await.is_ok() {
            if *receiver.borrow() == RTCIceGathererState::Complete {
                break;
            }
        }
    };

    if let Err(err) = timeout(Duration::from_secs(1), gather_candidates).await {
        debug!("Maximum time for gathering candidate has elapsed: {err:?}");
    }
}
