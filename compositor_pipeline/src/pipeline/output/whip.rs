use compositor_render::OutputId;
use crossbeam_channel::Receiver;
use payloader::DataKind;
use reqwest::Url;
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
    rtp_transceiver::{
        rtp_codec::{RTCRtpCodecCapability, RTCRtpCodecParameters, RTPCodecType},
        rtp_transceiver_direction::RTCRtpTransceiverDirection,
    },
    track::track_local::{track_local_static_rtp::TrackLocalStaticRTP, TrackLocalWriter},
};

use crate::{
    error::OutputInitError,
    event::Event,
    pipeline::{types::EncoderOutputEvent, AudioCodec, PipelineCtx, VideoCodec},
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
    pub video: Option<VideoCodec>,
    pub audio: Option<AudioCodec>,
}

impl WhipSender {
    pub fn new(
        output_id: &OutputId,
        options: WhipSenderOptions,
        packets_receiver: Receiver<EncoderOutputEvent>,
        pipeline_ctx: &PipelineCtx,
    ) -> Result<Self, OutputInitError> {
        let payloader = Payloader::new(options.video, options.audio);
        let packet_stream = PacketStream::new(packets_receiver, payloader, 1200);

        let should_close = Arc::new(AtomicBool::new(false));
        let endpoint_url = options.endpoint_url.clone();
        let output_id = output_id.clone();
        let should_close2 = should_close.clone();
        let event_emitter = pipeline_ctx.event_emitter.clone();
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
                start_whip_sender_thread(endpoint_url, should_close2, packet_stream, tokio_rt);
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
    // TODO send delete request to whip server
    fn drop(&mut self) {
        self.should_close
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

fn start_whip_sender_thread(
    endpoint_url: String,
    should_close: Arc<AtomicBool>,
    packet_stream: PacketStream,
    tokio_rt: Arc<tokio::runtime::Runtime>,
) {
    tokio_rt.block_on(async {
        let (peer_connection, video_track, audio_track) = init_pc().await;
        connect(
            peer_connection,
            endpoint_url,
            should_close,
            tokio_rt.clone(),
        )
        .await;

        for chunk in packet_stream {
            let chunk = match chunk {
                Ok(chunk) => chunk,
                Err(err) => {
                    error!("Failed to payload a packet: {}", err);
                    continue;
                }
            };

            match chunk.kind {
                DataKind::Audio => {
                    if let Err(_) = audio_track.write(&chunk.data).await {
                        error!("Error occurred while writing to audio track for session");
                    }
                }
                DataKind::Video => {
                    if let Err(_) = video_track.write(&chunk.data).await {
                        error!("Error occurred while writing to video track for session");
                    }
                }
            }
        }
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
            payload_type: 111,
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
    // Create Track that we send video back to browser on
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
    _should_close: Arc<AtomicBool>, // TODO handle should_close if necessary
    tokio_rt: Arc<tokio::runtime::Runtime>,
) {
    let (done_tx, _done_rx) = std::sync::mpsc::channel::<()>();

    peer_connection.on_ice_connection_state_change(Box::new(
        move |connection_state: RTCIceConnectionState| {
            debug!("Connection State has changed {connection_state}");
            if connection_state == RTCIceConnectionState::Connected {
                debug!("ice connected");
            } else if connection_state == RTCIceConnectionState::Failed {
                debug!("Done writing media files");
                let _ = done_tx.send(());
                // TODO handle failed connection properly
            }
            Box::pin(async {})
        },
    ));

    let offer = peer_connection.create_offer(None).await.unwrap();
    let client = reqwest::Client::new();

    info!("[WHIP] endpoint url: {}", endpoint_url);

    let response = client
        .post(endpoint_url)
        .header("Content-Type", "application/sdp")
        .body(offer.sdp.clone())
        .send()
        .await
        .unwrap();

    info!("[WHIP] response: {:?}", &response);

    let location = Url::try_from(
        response
            .headers()
            .get("location")
            .unwrap()
            .to_str()
            .unwrap(),
    )
    .unwrap();

    let answer = response.bytes().await.unwrap();
    let _ = peer_connection.set_local_description(offer).await.unwrap();

    peer_connection
        .set_remote_description(
            RTCSessionDescription::answer(std::str::from_utf8(&answer).unwrap().to_string())
                .unwrap(),
        )
        .await
        .unwrap();

    let client = Arc::new(client);

    peer_connection.on_ice_candidate(Box::new(move |candidate| {
        if let Some(candidate) = candidate {
            let client_clone = client.clone();
            let location_clone = location.clone();
            tokio_rt.spawn(async move {
                let ice_candidate = candidate.to_json().unwrap();
                let patch_response = client_clone
                    .patch(location_clone)
                    .header("Content-type", "application/trickle-ice-sdpfrag")
                    .body(serde_json::to_string(&ice_candidate).unwrap())
                    .send()
                    .await
                    .unwrap();
                println!("patch response: {patch_response:?}");
            });
        }
        Box::pin(async {})
    }));
}
