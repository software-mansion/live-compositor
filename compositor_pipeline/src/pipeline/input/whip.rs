use std::{net::SocketAddr, sync::Arc};

use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, Response, StatusCode},
    response::IntoResponse,
    routing::{delete, options, patch, post},
    Router,
};
use config::read_config;
use logger::init_logger;
use serde_json::{json, Value};
use tracing::{error, info};
use webrtc::{
    api::{
        interceptor_registry::register_default_interceptors,
        media_engine::{MediaEngine, MIME_TYPE_H264, MIME_TYPE_OPUS},
        APIBuilder,
    },
    ice_transport::{ice_candidate::RTCIceCandidateInit, ice_server::RTCIceServer},
    interceptor::registry::Registry,
    peer_connection::{
        configuration::RTCConfiguration, sdp::session_description::RTCSessionDescription,
        RTCPeerConnection,
    },
    rtp_transceiver::{
        rtp_codec::{RTCRtpCodecCapability, RTCRtpCodecParameters, RTPCodecType},
        rtp_transceiver_direction::RTCRtpTransceiverDirection,
        RTCRtpTransceiverInit,
    },
};

use webrtc_util::{Marshal, MarshalSize};

use std::sync::atomic::AtomicBool;

use crate::{
    pipeline::{
        decoder::{self},
        encoder,
        rtp::{BindToPortError, RequestedPort, TransportProtocol},
        types::{EncodedChunk, EncodedChunkKind},
    },
    queue::PipelineEvent,
};
use compositor_render::InputId;
use crossbeam_channel::{bounded, Receiver, Sender};
use rtcp::header::PacketType;
use tracing::{debug, span, warn, Level};
use webrtc_util::Unmarshal;

// use crate::pipeline::input::rtp::depayloader::{Depayloader, DepayloaderNewError};

use super::{AudioInputReceiver, Input, InputInitInfo, InputInitResult, VideoInputReceiver};
use tower_http::cors::{Any, CorsLayer};

mod config;
// mod depayloader;
mod logger;


use tokio::sync::{mpsc, Mutex, Notify};

pub struct WhipWhepState {
    pub whip: Arc<WhipUtils>,
    pub notifier: Arc<Notify>,
    pub video_receiver: Mutex<mpsc::Receiver<Vec<u8>>>,
    pub audio_receiver: Mutex<mpsc::Receiver<Vec<u8>>>,
}

pub async fn init() -> Arc<WhipWhepState> {
    let (_video_sender, video_receiver) = mpsc::channel(32);
    let (_audio_sender, audio_receiver) = mpsc::channel(32);

    Arc::new(WhipWhepState {
        whip: init_pc().await,
        notifier: Arc::new(Notify::new()),
        video_receiver: Mutex::new(video_receiver),
        audio_receiver: Mutex::new(audio_receiver),
    })
}

#[derive(Debug, thiserror::Error)]
pub enum WhipReceiverError {
    #[error("Error while setting socket options.")]
    SocketOptions(#[source] std::io::Error),

    #[error("Error while binding the socket.")]
    SocketBind(#[source] std::io::Error),

    #[error("Failed to register input. Port: {0} is already used or not available.")]
    PortAlreadyInUse(u16),

    #[error("Failed to register input. All ports in range {lower_bound} to {upper_bound} are already used or not available.")]
    AllPortsAlreadyInUse { lower_bound: u16, upper_bound: u16 },

    // #[error(transparent)]
    // DepayloaderError(#[from] DepayloaderNewError),
}

#[derive(Debug, Clone)]
pub struct WhipReceiverOptions {
    pub port: RequestedPort,
    pub transport_protocol: TransportProtocol,
    pub stream: RtpStream,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputVideoStream {
    pub options: decoder::VideoDecoderOptions,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputAudioStream {
    pub options: decoder::AudioDecoderOptions,
}

pub struct OutputAudioStream {
    pub options: encoder::EncoderOptions,
    pub payload_type: u8,
}

#[derive(Debug, Clone)]
pub struct RtpStream {
    pub video: Option<InputVideoStream>,
    pub audio: Option<InputAudioStream>,
}

struct DepayloaderThreadReceivers {
    video: Option<Receiver<PipelineEvent<EncodedChunk>>>,
    audio: Option<Receiver<PipelineEvent<EncodedChunk>>>,
}
pub struct WhipReceiver {
    should_close: Arc<AtomicBool>,
    pub token: String,
}

#[tokio::main]
async fn start_server() {
    let config = read_config();
    init_logger(config.logger.clone());
    let port = config.api_port;

    let state = init().await;

    let app = Router::new()
        // .route("/status", get(status))
        // .route("/whep", post(handle_whep))
        .route("/whip", post(handle_whip))
        .route("/whip", options(handle_options))
        .route("/session", patch(whip_ice_candidates_handler))
        .route("/session", delete(terminate_whip_session))
        // .route("/resource/:id", patch(whep_ice_candidates_handler))
        // .route("/resource/:id", delete(terminate_whep_session))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], port)))
        .await
        .unwrap();
    info!("started http server");
    axum::serve(listener, app).await.unwrap();
}

impl WhipReceiver {
    pub(super) fn start_new_input(
        input_id: &InputId,
        opts: WhipReceiverOptions,
    ) -> Result<InputInitResult, WhipReceiverError> {
        let should_close = Arc::new(AtomicBool::new(false));

        let packets_rx = todo!();

        let depayloader = todo!();

        // let depayloader_receivers =
            // Self::start_depayloader_thread(input_id, packets_rx, depayloader);

        // let video = match (depayloader_receivers.video, opts.stream.video) {
        //     (Some(chunk_receiver), Some(stream)) => Some(VideoInputReceiver::Encoded {
        //         chunk_receiver,
        //         decoder_options: stream.options,
        //     }),
        //     _ => None,
        // };
        // let audio = match (depayloader_receivers.audio, opts.stream.audio) {
        //     (Some(chunk_receiver), Some(stream)) => Some(AudioInputReceiver::Encoded {
        //         chunk_receiver,
        //         decoder_options: stream.options,
        //     }),
        //     _ => None,
        // };

        // Ok(InputInitResult {
        //     input: Input::Whip(Self {
        //         should_close,
        //         token: todo!(),
        //     }),
        //     video,
        //     audio,
        //     init_info: InputInitInfo { port: todo!()},
        // })
    }

    fn start_depayloader_thread(
        input_id: &InputId,
        receiver: Receiver<bytes::Bytes>,
        // depayloader: Depayloader,
    ) //-> DepayloaderThreadReceivers {
    {
        // let (video_sender, video_receiver) = depayloader
        //     .video
        //     .as_ref()
        //     .map(|_| bounded(5))
        //     .map_or((None, None), |(tx, rx)| (Some(tx), Some(rx)));
        // let (audio_sender, audio_receiver) = depayloader
        //     .audio
        //     .as_ref()
        //     .map(|_| bounded(5))
        //     .map_or((None, None), |(tx, rx)| (Some(tx), Some(rx)));

        let input_id = input_id.clone();
        std::thread::Builder::new()
            .name(format!("Depayloading thread for input: {}", input_id.0))
            .spawn(move || {
                let _span = span!(
                    Level::INFO,
                    "RTP depayloader",
                    input_id = input_id.to_string()
                )
                .entered();
                // run_depayloader_thread(receiver, depayloader, video_sender, audio_sender)
            })
            .unwrap();

        // DepayloaderThreadReceivers {
        //     video: video_receiver,
        //     audio: audio_receiver,
        // }
    }
}

impl Drop for WhipReceiver {
    fn drop(&mut self) {
        self.should_close
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

fn run_depayloader_thread(
    receiver: Receiver<bytes::Bytes>,
    // mut depayloader: Depayloader,
    video_sender: Option<Sender<PipelineEvent<EncodedChunk>>>,
    audio_sender: Option<Sender<PipelineEvent<EncodedChunk>>>,
) {
    let mut audio_eos_received = audio_sender.as_ref().map(|_| false);
    let mut video_eos_received = video_sender.as_ref().map(|_| false);
    let mut audio_ssrc = None;
    let mut video_ssrc = None;

    let mut maybe_send_video_eos = || {
        if let (Some(sender), Some(false)) = (&video_sender, video_eos_received) {
            video_eos_received = Some(true);
            if sender.send(PipelineEvent::EOS).is_err() {
                debug!("Failed to send EOS from RTP video depayloader. Channel closed.");
            }
        }
    };
    let mut maybe_send_audio_eos = || {
        if let (Some(sender), Some(false)) = (&audio_sender, audio_eos_received) {
            audio_eos_received = Some(true);
            if sender.send(PipelineEvent::EOS).is_err() {
                debug!("Failed to send EOS from RTP audio depayloader. Channel closed.");
            }
        }
    };
    loop {
        let Ok(mut buffer) = receiver.recv() else {
            debug!("Closing RTP depayloader thread.");
            break;
        };

        match rtp::packet::Packet::unmarshal(&mut buffer.clone()) {
            // https://datatracker.ietf.org/doc/html/rfc5761#section-4
            //
            // Given these constraints, it is RECOMMENDED to follow the guidelines
            // in the RTP/AVP profile [7] for the choice of RTP payload type values,
            // with the additional restriction that payload type values in the range
            // 64-95 MUST NOT be used.
            Ok(packet) if packet.header.payload_type < 64 || packet.header.payload_type > 95 => {
                if packet.header.payload_type == 96 && video_ssrc.is_none() {
                    video_ssrc = Some(packet.header.ssrc);
                }
                if packet.header.payload_type == 97 && audio_ssrc.is_none() {
                    audio_ssrc = Some(packet.header.ssrc);
                }

                // match depayloader.depayload(packet) {
                //     Ok(chunks) => {
                //         for chunk in chunks {
                //             match &chunk.kind {
                //                 EncodedChunkKind::Video(_) => {
                //                     video_sender.as_ref().map(|video_sender| {
                //                         video_sender.send(PipelineEvent::Data(chunk))
                //                     })
                //                 }
                //                 EncodedChunkKind::Audio(_) => {
                //                     audio_sender.as_ref().map(|audio_sender| {
                //                         audio_sender.send(PipelineEvent::Data(chunk))
                //                     })
                //                 }
                //             };
                //         }
                //     }
                //     Err(err) => {
                //         warn!("RTP depayloading error: {}", err);
                //         continue;
                //     }
                // }
            }
            Ok(_) | Err(_) => {
                match rtcp::packet::unmarshal(&mut buffer) {
                    Ok(rtcp_packets) => {
                        for rtcp_packet in rtcp_packets {
                            if let PacketType::Goodbye = rtcp_packet.header().packet_type {
                                for ssrc in rtcp_packet.destination_ssrc() {
                                    if Some(ssrc) == audio_ssrc {
                                        maybe_send_audio_eos()
                                    }
                                    if Some(ssrc) == video_ssrc {
                                        maybe_send_video_eos()
                                    }
                                }
                            } else {
                                debug!(
                                    packet_type=?rtcp_packet.header().packet_type,
                                    "Received RTCP packet"
                                )
                            }
                        }
                    }
                    Err(err) => {
                        warn!(%err, "Received an unexpected packet, which is not recognized either as RTP or RTCP. Dropping.");
                    }
                }
                continue;
            }
        };
    }
    maybe_send_audio_eos();
    maybe_send_video_eos();
}

#[derive(Debug, thiserror::Error)]
pub enum DepayloadingError {
    #[error("Bad payload type {0}")]
    BadPayloadType(u8),
    #[error(transparent)]
    Rtp(#[from] rtp::Error),
    // #[error("AAC depayoading error")]
    // Aac(#[from] depayloader::AacDepayloadingError),
}

impl From<BindToPortError> for WhipReceiverError {
    fn from(value: BindToPortError) -> Self {
        match value {
            BindToPortError::SocketBind(err) => WhipReceiverError::SocketBind(err),
            BindToPortError::PortAlreadyInUse(port) => WhipReceiverError::PortAlreadyInUse(port),
            BindToPortError::AllPortsAlreadyInUse {
                lower_bound,
                upper_bound,
            } => WhipReceiverError::AllPortsAlreadyInUse {
                lower_bound,
                upper_bound,
            },
        }
    }
}

#[derive(Clone)]
pub struct WhipUtils {
    pub peer_connection: Arc<RTCPeerConnection>,
}

pub async fn init_pc() -> Arc<WhipUtils> {
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
            payload_type: 96,
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
            payload_type: 97,
            ..Default::default()
        },
        RTPCodecType::Audio,
    )
    .unwrap();

    // Create a InterceptorRegistry. This is the user configurable RTP/RTCP Pipeline.
    // This provides NACKs, RTCP Reports and other features. If you use `webrtc.NewPeerConnection`
    // this is enabled by default. If you are manually managing You MUST create a InterceptorRegistry
    // for each PeerConnection.
    let mut registry = Registry::new();

    // Use the default set of Interceptors
    registry = register_default_interceptors(registry, &mut m).unwrap();

    // Create the API object with the MediaEngine
    let api = APIBuilder::new()
        .with_media_engine(m)
        .with_interceptor_registry(registry)
        .build();

    // Prepare the configuration
    let config = RTCConfiguration {
        ice_servers: vec![RTCIceServer {
            urls: vec!["stun:stun.l.google.com:19302".to_owned()],
            ..Default::default()
        }],
        ..Default::default()
    };

    // Create a new RTCPeerConnection
    let peer_connection = Arc::new(api.new_peer_connection(config).await.unwrap());

    // Allow us to receive 1 audio track, and 1 video track
    peer_connection
        .add_transceiver_from_kind(
            RTPCodecType::Audio,
            Some(RTCRtpTransceiverInit {
                direction: RTCRtpTransceiverDirection::Recvonly,
                send_encodings: vec![],
            }),
        )
        .await
        .unwrap();
    peer_connection
        .add_transceiver_from_kind(
            RTPCodecType::Video,
            Some(RTCRtpTransceiverInit {
                direction: RTCRtpTransceiverDirection::Recvonly,
                send_encodings: vec![],
            }),
        )
        .await
        .unwrap();

    Arc::new(WhipUtils { peer_connection })
}

pub async fn handle_whip(
    State(state): State<Arc<WhipWhepState>>,
    headers: HeaderMap,
    offer: String,
) -> Response<Body> {
    info!("[whip] got headers: {headers:?}");
    info!("[whip] got request: {offer}");

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

    let peer_connection = state.whip.peer_connection.clone();

    peer_connection.on_track(Box::new(move |track, _, _| {
        // let video_sender_clone = state.video_sender.clone();
        // let audio_sender_clone = state.audio_sender.clone();

        tokio::spawn(async move {
            let track_kind = track.kind();

            while let Ok((rtp_packet, _)) = track.read_rtp().await {
                let mut buf = vec![0u8; rtp_packet.marshal_size()];
                rtp_packet.marshal_to(&mut buf).unwrap();

                // Send RTP packets through the channel
                // if track_kind == RTPCodecType::Audio {
                //     if let Err(e) = audio_sender_clone.send(buf).await {
                //         error!("Failed to send audio RTP packet: {e}");
                //     }
                // } else if track_kind == RTPCodecType::Video {
                //     if let Err(e) = video_sender_clone.send(buf).await {
                //         error!("Failed to send video RTP packet: {e}");
                //     }
                // }
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

    let candidate: Value = serde_json::from_str(&candidate).unwrap();

    let candidate_str = candidate["candidate"].as_str().unwrap_or("");
    let candidate_obj = RTCIceCandidateInit {
        candidate: candidate_str.to_string(),
        sdp_mid: candidate["sdpMid"].as_str().map(|s| s.to_string()),
        sdp_mline_index: candidate["sdpMLineIndex"].as_u64().map(|i| i as u16),
        ..Default::default()
    };

    let _ = state
        .whip
        .peer_connection
        .add_ice_candidate(candidate_obj)
        .await;

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

    if let Err(err) = state.whip.peer_connection.close().await {
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
