use crate::pipeline::{
    input::whip::handle_track,
    whip_whep::{
        error::WhipServerError, init_peer_connection, validate_bearer_token::validate_token,
        WhipInputConnectionOptions, WhipWhepState,
    },
};
use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, Response, StatusCode},
};
use compositor_render::InputId;
use std::{sync::Arc, time::Duration};
use tokio::{sync::watch, time::timeout};
use tracing::{debug, error, info, warn};
use urlencoding::encode;
use webrtc::{
    ice_transport::{
        ice_gatherer_state::RTCIceGathererState, ice_gathering_state::RTCIceGatheringState,
    },
    peer_connection::{
        peer_connection_state::RTCPeerConnectionState,
        sdp::session_description::RTCSessionDescription, RTCPeerConnection,
    },
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

    let input_components = get_whip_input_connection_options(state.clone(), input_id.clone())?;

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
    )
    .await?;

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
        let state_clone = state.clone();
        let input_id_clone = input_id.clone();
        let video_sender_clone = input_components.video_sender.clone();
        let audio_sender_clone = input_components.audio_sender.clone();
        let depayloader_clone = input_components.depayloader.clone();

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
            "Read local description error".to_string(),
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

fn get_whip_input_connection_options(
    state: Arc<WhipWhepState>,
    input_id: InputId,
) -> Result<WhipInputConnectionOptions, WhipServerError> {
    if let Ok(connections) = state.input_connections.lock() {
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
    } else {
        Err(WhipServerError::InternalError(
            "Input connections lock error".to_string(),
        ))
    }
}

pub fn validate_sdp_content_type(headers: &HeaderMap) -> Result<(), WhipServerError> {
    if let Some(content_type) = headers.get("Content-Type") {
        if content_type.as_bytes() != b"application/sdp" {
            error!("Invalid Content-Type, expecting application/sdp");
            return Err(WhipServerError::InternalError(
                "Invalid Content-Type".to_string(),
            ));
        }
    } else {
        error!("Missing Content-Type header");
        return Err(WhipServerError::BadRequest(
            "Missing Content-Type header".to_string(),
        ));
    }
    Ok(())
}

pub async fn gather_ice_candidates_for_one_second(peer_connection: Arc<RTCPeerConnection>) {
    let (tx, mut rx) = watch::channel(peer_connection.ice_gathering_state());

    peer_connection.on_ice_gathering_state_change(Box::new(move |gatherer_state| {
        let gathering_state = match gatherer_state {
            RTCIceGathererState::Complete => RTCIceGatheringState::Complete,
            RTCIceGathererState::Unspecified => RTCIceGatheringState::Unspecified,
            RTCIceGathererState::New => RTCIceGatheringState::New,
            RTCIceGathererState::Gathering => RTCIceGatheringState::Gathering,
            RTCIceGathererState::Closed => RTCIceGatheringState::Unspecified,
        };
        if let Err(err) = tx.send(gathering_state) {
            debug!("Cannot send gathering_state: {err:?}");
        };
        Box::pin(async {})
    }));

    let gather_candidates = async {
        while rx.changed().await.is_ok() {
            if *rx.borrow() == RTCIceGatheringState::Complete {
                break;
            }
        }
    };

    if let Err(err) = timeout(Duration::from_secs(1), gather_candidates).await {
        debug!("Maximum time for gathering candidate has elapsed: {err:?}");
    }
}
