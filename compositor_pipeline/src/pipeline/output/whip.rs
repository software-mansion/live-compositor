use compositor_render::OutputId;
use crossbeam_channel::{Receiver, Sender};
use payloader::Payload;
use reqwest::{header::HeaderMap, Url};
use std::sync::{atomic::AtomicBool, Arc};
use tracing::{debug, error, info, span, Level};
use webrtc::{
    api::{
        interceptor_registry::register_default_interceptors,
        media_engine::{MediaEngine, MIME_TYPE_H264, MIME_TYPE_OPUS},
        APIBuilder,
    },
    ice_transport::{ice_connection_state::RTCIceConnectionState, ice_server::RTCIceServer},
    interceptor::registry::Registry,
    peer_connection::{
        configuration::RTCConfiguration, sdp::session_description::RTCSessionDescription,
        RTCPeerConnection,
    },
    rtcp::payload_feedbacks::picture_loss_indication::PictureLossIndication,
    rtp_transceiver::{
        rtp_codec::{RTCRtpCodecCapability, RTCRtpCodecParameters, RTPCodecType},
        rtp_transceiver_direction::RTCRtpTransceiverDirection,
    },
    track::track_local::{track_local_static_rtp::TrackLocalStaticRTP, TrackLocalWriter},
};

use crate::{
    error::OutputInitError,
    event::Event,
    pipeline::{
        types::EncoderOutputEvent,
        whip_whep::{AUDIO_PAYLOAD_TYPE, VIDEO_PAYLOAD_TYPE},
        AudioCodec, PipelineCtx, VideoCodec,
    },
};

use self::{packet_stream::PacketStream, payloader::Payloader};

mod packet_stream;
mod payloader;

#[derive(Debug)]
pub struct WhipSender {
    pub connection_options: WhipSenderOptions,
    should_close: Arc<AtomicBool>,
}

#[derive(Debug, Clone)]
pub struct WhipSenderOptions {
    pub endpoint_url: String,
    pub bearer_token: Option<String>,
    pub video: Option<VideoCodec>,
    pub audio: Option<AudioCodec>,
}

#[derive(Debug, thiserror::Error)]
pub enum WhipError {
    #[error("Missing location header in WHIP response")]
    MissingLocationHeader,
}

impl WhipSender {
    pub fn new(
        output_id: &OutputId,
        options: WhipSenderOptions,
        packets_receiver: Receiver<EncoderOutputEvent>,
        request_keyframe_sender: Option<Sender<()>>,
        pipeline_ctx: &PipelineCtx,
    ) -> Result<Self, OutputInitError> {
        let payloader = Payloader::new(options.video, options.audio);
        let packet_stream = PacketStream::new(packets_receiver, payloader, 1400);

        let should_close = Arc::new(AtomicBool::new(false));
        let endpoint_url = options.endpoint_url.clone();
        let bearer_token = options.bearer_token.clone();
        let output_id = output_id.clone();
        let should_close2 = should_close.clone();
        let event_emitter = pipeline_ctx.event_emitter.clone();
        let request_keyframe_sender = request_keyframe_sender.clone();
        let tokio_rt = pipeline_ctx.tokio_rt.clone();

        std::thread::Builder::new()
            .name(format!("WHIP sender for output {}", output_id))
            .spawn(move || {
                let _span = span!(
                    Level::INFO,
                    "WHIP sender",
                    output_id = output_id.to_string()
                )
                .entered();
                start_whip_sender_thread(
                    endpoint_url,
                    bearer_token,
                    should_close2,
                    packet_stream,
                    request_keyframe_sender,
                    tokio_rt,
                );
                event_emitter.emit(Event::OutputDone(output_id));
                debug!("Closing WHIP sender thread.")
            })
            .unwrap();

        Ok(Self {
            connection_options: options,
            should_close,
        })
    }
}

impl Drop for WhipSender {
    fn drop(&mut self) {
        self.should_close
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

fn start_whip_sender_thread(
    endpoint_url: String,
    bearer_token: Option<String>,
    should_close: Arc<AtomicBool>,
    packet_stream: PacketStream,
    request_keyframe_sender: Option<Sender<()>>,
    tokio_rt: Arc<tokio::runtime::Runtime>,
) {
    tokio_rt.block_on(async {
        let client = reqwest::Client::new();
        let (peer_connection, video_track, audio_track) = init_pc().await;
        let whip_session_url = match connect(
            peer_connection,
            endpoint_url,
            bearer_token,
            should_close.clone(),
            tokio_rt.clone(),
            client.clone(),
            request_keyframe_sender,
        )
        .await
        {
            Ok(val) => val,
            Err(_) => return,
        };

        for chunk in packet_stream {
            if should_close.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }
            let chunk = match chunk {
                Ok(chunk) => chunk,
                Err(err) => {
                    error!("Failed to payload a packet: {}", err);
                    continue;
                }
            };
            // info!("{:?}", chunk);

            match chunk {
                Payload::Video(bytes) => {
                    // info!("Video: {:?}", bytes);
                    if video_track.write(&bytes).await.is_err() {
                        error!("Error occurred while writing to video track for session");
                    }
                }
                Payload::Audio(bytes) => {
                    // info!("Audio: {:?}", bytes);
                    if audio_track.write(&bytes).await.is_err() {
                        error!("Error occurred while writing to audio track for session");
                    }
                }
            }
        }
        let _ = client.delete(whip_session_url).send().await;
    });
}

async fn init_pc() -> (
    Arc<RTCPeerConnection>,
    Arc<TrackLocalStaticRTP>,
    Arc<TrackLocalStaticRTP>,
) {
    let mut m = MediaEngine::default();
    m.register_default_codecs().unwrap();
    m.register_codec(
        RTCRtpCodecParameters {
            capability: RTCRtpCodecCapability {
                mime_type: MIME_TYPE_H264.to_owned(),
                clock_rate: 90000,
                channels: 0,
                sdp_fmtp_line: "".to_owned(),
                rtcp_feedback: vec![],
            },
            payload_type: VIDEO_PAYLOAD_TYPE,
            ..Default::default()
        },
        RTPCodecType::Video,
    )
    .unwrap();
    m.register_codec(
        RTCRtpCodecParameters {
            capability: RTCRtpCodecCapability {
                mime_type: MIME_TYPE_OPUS.to_owned(),
                clock_rate: 48000,
                channels: 2,
                sdp_fmtp_line: "".to_owned(),
                rtcp_feedback: vec![],
            },
            payload_type: AUDIO_PAYLOAD_TYPE,
            ..Default::default()
        },
        RTPCodecType::Audio,
    )
    .unwrap();
    let mut registry = Registry::new();
    registry = register_default_interceptors(registry, &mut m).unwrap();
    let api = APIBuilder::new()
        .with_media_engine(m)
        .with_interceptor_registry(registry)
        .build();

    let config = RTCConfiguration {
        ice_servers: vec![RTCIceServer {
            urls: vec!["stun:stun.l.google.com:19302".to_owned()],
            ..Default::default()
        }],
        ..Default::default()
    };
    let peer_connection = Arc::new(api.new_peer_connection(config).await.unwrap());
    let video_track = Arc::new(TrackLocalStaticRTP::new(
        RTCRtpCodecCapability {
            mime_type: MIME_TYPE_H264.to_owned(),
            ..Default::default()
        },
        "video".to_owned(),
        "webrtc-rs".to_owned(),
    ));
    let audio_track = Arc::new(TrackLocalStaticRTP::new(
        RTCRtpCodecCapability {
            mime_type: MIME_TYPE_OPUS.to_owned(),
            ..Default::default()
        },
        "audio".to_owned(),
        "webrtc-rs".to_owned(),
    ));
    let _ = peer_connection.add_track(video_track.clone()).await;
    let _ = peer_connection.add_track(audio_track.clone()).await;
    let transceivers = peer_connection.get_transceivers().await;
    for transceiver in transceivers {
        transceiver
            .set_direction(RTCRtpTransceiverDirection::Sendonly)
            .await;
    }
    (peer_connection, video_track, audio_track)
}

async fn connect(
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
            debug!("Connection State has changed {connection_state}");
            if connection_state == RTCIceConnectionState::Connected {
                debug!("ice connected");
            } else if connection_state == RTCIceConnectionState::Failed {
                debug!("Done writing media files");
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
                    let packets = &sender.read_rtcp().await.unwrap().0;
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
                }
            });
        }
    }

    let offer = peer_connection.create_offer(None).await.unwrap();

    info!("[WHIP] endpoint url: {}", endpoint_url);

    let mut header_map = HeaderMap::new();
    header_map.append("Content-Type", "application/sdp".parse().unwrap());

    let bearer_token = bearer_token.map(Arc::new);

    if let Some(token) = bearer_token.clone() {
        header_map.append("Authorization", format!("Bearer {token}").parse().unwrap());
    }

    let response = client
        .post(endpoint_url.clone())
        .headers(header_map)
        .body(offer.sdp.clone())
        .send()
        .await
        .unwrap();

    info!("[WHIP] response: {:?}", &response);

    let parsed_endpoint_url = Url::parse(&endpoint_url).unwrap();

    let location_url_path = response.headers()
        .get("location")
        .and_then(|url| url.to_str().ok())
        .ok_or_else(|| {
            error!("Unable to get location endpoint, check correctness of WHIP endpoint and your Bearer token");
            WhipError::MissingLocationHeader
        })?;

    let location_url = Url::try_from(
        format!(
            "{}://{}:{}{}",
            parsed_endpoint_url.scheme(),
            parsed_endpoint_url.host_str().unwrap(),
            parsed_endpoint_url.port().unwrap(),
            location_url_path
        )
        .as_str(),
    )
    .unwrap();

    let answer = response.bytes().await.unwrap();
    peer_connection.set_local_description(offer).await.unwrap();

    peer_connection
        .set_remote_description(
            RTCSessionDescription::answer(std::str::from_utf8(&answer).unwrap().to_string())
                .unwrap(),
        )
        .await
        .unwrap();

    let client = Arc::new(client);

    let location1: Url = location_url.clone();

    peer_connection.on_ice_candidate(Box::new(move |candidate| {
        if let Some(candidate) = candidate {
            let client_clone = client.clone();
            let location2 = location1.clone();
            let bearer_token1 = bearer_token.clone();
            tokio_rt.spawn(async move {
                let ice_candidate = candidate.to_json().unwrap();

                let mut header_map = HeaderMap::new();
                header_map.append(
                    "Content-Type",
                    "application/trickle-ice-sdpfrag".parse().unwrap(),
                );

                if let Some(token) = bearer_token1 {
                    header_map.append("Authorization", format!("Bearer {token}").parse().unwrap());
                }

                let _ = client_clone
                    .patch(location2)
                    .headers(header_map)
                    .body(serde_json::to_string(&ice_candidate).unwrap())
                    .send()
                    .await
                    .unwrap();
            });
        }
        Box::pin(async {})
    }));

    Ok(location_url.clone())
}
