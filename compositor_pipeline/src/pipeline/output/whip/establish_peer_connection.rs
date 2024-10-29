use crossbeam_channel::Sender;

use super::WhipError;
use reqwest::{header::HeaderMap, Client, Method, StatusCode, Url};
use std::sync::{atomic::AtomicBool, Arc};
use tracing::{debug, error, info};
use webrtc::{
    ice_transport::{ice_candidate::RTCIceCandidate, ice_connection_state::RTCIceConnectionState},
    peer_connection::{sdp::session_description::RTCSessionDescription, RTCPeerConnection},
    rtcp::payload_feedbacks::picture_loss_indication::PictureLossIndication,
};

pub async fn connect(
    peer_connection: Arc<RTCPeerConnection>,
    endpoint_url: String,
    bearer_token: Option<String>,
    should_close: Arc<AtomicBool>,
    tokio_rt: Arc<tokio::runtime::Runtime>,
    client: reqwest::Client,
    request_keyframe_sender: Option<Sender<()>>,
) -> Result<Url, WhipError> {
    peer_connection.on_ice_connection_state_change(Box::new(
        move |connection_state: RTCIceConnectionState| {
            debug!("Connection State has changed {connection_state}.");
            if connection_state == RTCIceConnectionState::Connected {
                debug!("Ice connected.");
            } else if connection_state == RTCIceConnectionState::Failed {
                debug!("Ice connection failed.");
                should_close.store(true, std::sync::atomic::Ordering::Relaxed);
            }
            Box::pin(async {})
        },
    ));

    if let Some(keyframe_sender) = request_keyframe_sender {
        let senders = peer_connection.get_senders().await;
        for sender in senders {
            let keyframe_sender_clone = keyframe_sender.clone();
            tokio_rt.spawn(async move {
                loop {
                    if let Ok((packets, _)) = &sender.read_rtcp().await {
                        for packet in packets {
                            if packet
                                .as_any()
                                .downcast_ref::<PictureLossIndication>()
                                .is_some()
                            {
                                if let Err(err) = keyframe_sender_clone.send(()) {
                                    debug!(%err, "Failed to send keyframe request to the encoder.");
                                };
                            }
                        }
                    } else {
                        debug!("Failed to read RTCP packets from the sender.");
                    }
                }
            });
        }
    }

    let offer = peer_connection
        .create_offer(None)
        .await
        .map_err(WhipError::OfferCreationError)?;

    let endpoint_url = Url::parse(&endpoint_url)
        .map_err(|e| WhipError::InvalidEndpointUrl(e, endpoint_url.clone()))?;

    info!("Endpoint url: {}", endpoint_url);

    let mut header_map = HeaderMap::new();
    header_map.append("Content-Type", "application/sdp".parse().unwrap());

    let bearer_token = bearer_token.map(Arc::new);

    if let Some(token) = bearer_token.clone() {
        header_map.append("Authorization", format!("Bearer {token}").parse().unwrap());
    }

    let response = match client
        .post(endpoint_url.clone())
        .headers(header_map)
        .body(offer.sdp.clone())
        .send()
        .await
    {
        Ok(res) => res,
        Err(_) => return Err(WhipError::RequestFailed(Method::POST, endpoint_url)),
    };

    if response.status() >= StatusCode::BAD_REQUEST {
        let status = response.status();
        let answer = response
            .text()
            .await
            .map_err(|e| WhipError::BodyParsingError("sdp offer".to_string(), e))?;
        return Err(WhipError::BadStatus(status, answer));
    }

    let location_url_path = response
        .headers()
        .get("location")
        .and_then(|url| url.to_str().ok())
        .ok_or_else(|| WhipError::MissingLocationHeader)?;

    let scheme = endpoint_url.scheme();
    let host = endpoint_url
        .host_str()
        .ok_or_else(|| WhipError::MissingHost)?;

    let port = endpoint_url.port().ok_or_else(|| WhipError::MissingPort)?;

    let formatted_url = format!("{}://{}:{}{}", scheme, host, port, location_url_path);

    let location_url = Url::try_from(formatted_url.as_str())
        .map_err(|e| WhipError::InvalidEndpointUrl(e, formatted_url))?;

    peer_connection
        .set_local_description(offer)
        .await
        .map_err(WhipError::LocalDescriptionError)?;

    let answer = response
        .text()
        .await
        .map_err(|e| WhipError::BodyParsingError("sdp offer".to_string(), e))?;

    let rtc_answer =
        RTCSessionDescription::answer(answer).map_err(WhipError::RTCSessionDescriptionError)?;

    peer_connection
        .set_remote_description(rtc_answer)
        .await
        .map_err(WhipError::RemoteDescriptionError)?;

    let client = Arc::new(client);

    let location_clone: Url = location_url.clone();

    peer_connection.on_ice_candidate(Box::new(move |candidate| {
        if let Some(candidate) = candidate {
            let client_clone = client.clone();
            let location_clone = location_clone.clone();
            let bearer_token_clone = bearer_token.clone();
            tokio_rt.spawn(async move {
                if let Err(err) =
                    handle_candidate(candidate, bearer_token_clone, client_clone, location_clone)
                        .await
                {
                    error!("{err}");
                }
            });
        }
        Box::pin(async {})
    }));

    Ok(location_url.clone())
}

async fn handle_candidate(
    candidate: RTCIceCandidate,
    bearer_token: Option<Arc<String>>,
    client: Arc<Client>,
    location: Url,
) -> Result<(), WhipError> {
    let ice_candidate = candidate
        .to_json()
        .map_err(WhipError::IceCandidateToJsonError)?;

    let mut header_map = HeaderMap::new();
    header_map.append(
        "Content-Type",
        "application/trickle-ice-sdpfrag".parse().unwrap(),
    );

    if let Some(token) = bearer_token {
        header_map.append("Authorization", format!("Bearer {token}").parse().unwrap());
    }

    let response = match client
        .patch(location.clone())
        .headers(header_map)
        .body(serde_json::to_string(&ice_candidate)?)
        .send()
        .await
    {
        Ok(res) => res,
        Err(_) => return Err(WhipError::RequestFailed(Method::PATCH, location)),
    };

    if response.status() >= StatusCode::BAD_REQUEST {
        let status = response.status();
        let body_str = response
            .text()
            .await
            .map_err(|e| WhipError::BodyParsingError("Trickle ICE patch".to_string(), e))?;
        return Err(WhipError::BadStatus(status, body_str));
    };
    Ok(())
}
