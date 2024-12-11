use std::{sync::Arc, time::Duration};

use axum::http::HeaderValue;
use rand::{rngs::StdRng, Rng, SeedableRng};
use tokio::{
    sync::watch,
    time::{sleep, timeout},
};
use tracing::{debug, warn};
use webrtc::{
    ice_transport::{
        ice_candidate::RTCIceCandidateInit, ice_gatherer_state::RTCIceGathererState,
        ice_gathering_state::RTCIceGatheringState,
    },
    peer_connection::RTCPeerConnection,
};

use crate::pipeline::whip_whep::error::WhipServerError;

pub async fn validate_token(
    expected_token: Option<String>,
    auth_header_value: Option<&HeaderValue>,
) -> Result<(), WhipServerError> {
    match (expected_token, auth_header_value) {
        (Some(bearer_token), Some(auth_str)) => {
            let auth_str = auth_str.to_str().map_err(|_| {
                WhipServerError::Unauthorized("Invalid UTF-8 in header".to_string())
            })?;

            if let Some(token_from_header) = auth_str.strip_prefix("Bearer ") {
                if token_from_header == bearer_token {
                    Ok(())
                } else {
                    let mut rng = StdRng::from_entropy();
                    let millis = rng.gen_range(50..1000);
                    sleep(Duration::from_millis(millis)).await;
                    warn!("Invalid or mismatched token provided");
                    Err(WhipServerError::Unauthorized(
                        "Invalid or mismatched token provided".to_string(),
                    ))
                }
            } else {
                Err(WhipServerError::Unauthorized(
                    "Authorization header format incorrect".to_string(),
                ))
            }
        }
        _ => Err(WhipServerError::Unauthorized(
            "Expected token and authorization header required".to_string(),
        )),
    }
}

pub async fn gather_ice_candidates_for_one_second(pc: Arc<RTCPeerConnection>) {
    let (tx, mut rx) = watch::channel(pc.ice_gathering_state());

    pc.on_ice_gathering_state_change(Box::new(move |gatherer_state| {
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
