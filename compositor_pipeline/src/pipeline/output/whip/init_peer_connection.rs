use crate::{
    audio_mixer::AudioChannels,
    pipeline::{AudioCodec, VideoCodec},
};

use super::{WhipAudioOptions, WhipCtx, WhipError};
use std::sync::Arc;
use webrtc::{
    api::{
        interceptor_registry::register_default_interceptors,
        media_engine::{MediaEngine, MIME_TYPE_H264, MIME_TYPE_OPUS},
        APIBuilder,
    },
    ice_transport::ice_server::RTCIceServer,
    interceptor::registry::Registry,
    peer_connection::{configuration::RTCConfiguration, RTCPeerConnection},
    rtp_transceiver::{
        rtp_codec::{RTCRtpCodecCapability, RTCRtpCodecParameters, RTPCodecType},
        rtp_transceiver_direction::RTCRtpTransceiverDirection,
        RTCPFeedback,
    },
    track::track_local::track_local_static_rtp::TrackLocalStaticRTP,
};

pub async fn init_peer_connection(
    whip_ctx: &WhipCtx,
) -> Result<
    (
        Arc<RTCPeerConnection>,
        Option<Arc<TrackLocalStaticRTP>>,
        Option<Arc<TrackLocalStaticRTP>>,
    ),
    WhipError,
> {
    let mut media_engine = MediaEngine::default();

    register_codecs(&mut media_engine)?;

    //media_engine.register_default_codecs()?;
    //if let Some(video) = whip_ctx.options.video {
    //    media_engine.register_codec(video_codec_parameters(video), RTPCodecType::Video)?;
    //}
    //
    //if let Some(audio) = whip_ctx.options.audio {
    //    media_engine.register_codec(
    //        audio_codec_parameters(audio, whip_ctx.pipeline_ctx.mixing_sample_rate)?,
    //        RTPCodecType::Audio,
    //    )?;
    //}
    let mut registry = Registry::new();
    registry = register_default_interceptors(registry, &mut media_engine)?;
    let api = APIBuilder::new()
        .with_media_engine(media_engine)
        .with_interceptor_registry(registry)
        .build();

    let config = RTCConfiguration {
        ice_servers: vec![RTCIceServer {
            urls: whip_ctx.pipeline_ctx.stun_servers.to_vec(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let peer_connection = Arc::new(api.new_peer_connection(config).await?);

    let video_track = match whip_ctx.options.video {
        Some(video) => {
            let video_track = Arc::new(TrackLocalStaticRTP::new(
                video_codec_capability(video),
                "video".to_owned(),
                format!("live-compositor-{}-video", whip_ctx.output_id).to_owned(),
            ));
            peer_connection
                .add_track(video_track.clone())
                .await
                .map_err(WhipError::PeerConnectionInitError)?;
            Some(video_track)
        }
        None => None,
    };
    let audio_track = match whip_ctx.options.audio {
        Some(audio_options) => {
            let audio_track = Arc::new(TrackLocalStaticRTP::new(
                audio_codec_capability(audio_options, whip_ctx.pipeline_ctx.mixing_sample_rate)?,
                "audio".to_owned(),
                format!("live-compositor-{}-audio", whip_ctx.output_id).to_owned(),
            ));
            peer_connection
                .add_track(audio_track.clone())
                .await
                .map_err(WhipError::PeerConnectionInitError)?;
            Some(audio_track)
        }
        None => None,
    };
    let transceivers = peer_connection.get_transceivers().await;
    for transceiver in transceivers {
        transceiver
            .set_direction(RTCRtpTransceiverDirection::Sendonly)
            .await;
    }
    Ok((peer_connection, video_track, audio_track))
}

fn video_codec_capability(video: VideoCodec) -> RTCRtpCodecCapability {
    match video {
        VideoCodec::H264 => RTCRtpCodecCapability {
            mime_type: MIME_TYPE_H264.to_owned(),
            clock_rate: 90000,
            channels: 0,
            sdp_fmtp_line: "".to_owned(),
            rtcp_feedback: vec![],
        },
    }
}

fn audio_codec_capability(
    audio_options: WhipAudioOptions,
    sample_rate: u32,
) -> Result<RTCRtpCodecCapability, WhipError> {
    match audio_options.codec {
        AudioCodec::Opus => Ok(RTCRtpCodecCapability {
            mime_type: MIME_TYPE_OPUS.to_owned(),
            clock_rate: sample_rate,
            channels: match audio_options.channels {
                AudioChannels::Mono => 1,
                AudioChannels::Stereo => 2,
            },
            sdp_fmtp_line: "".to_owned(),
            rtcp_feedback: vec![],
        }),
        AudioCodec::Aac => Err(WhipError::UnsupportedCodec("AAC")),
    }
}

fn video_codec_parameters(video: VideoCodec) -> RTCRtpCodecParameters {
    let capability = video_codec_capability(video);
    let payload_type = match video {
        VideoCodec::H264 => 96,
    };
    RTCRtpCodecParameters {
        capability,
        payload_type,
        ..Default::default()
    }
}

fn audio_codec_parameters(
    audio_options: WhipAudioOptions,
    sample_rate: u32,
) -> Result<RTCRtpCodecParameters, WhipError> {
    let capability = audio_codec_capability(audio_options, sample_rate)?;
    let payload_type = match audio_options.codec {
        AudioCodec::Aac => return Err(WhipError::UnsupportedCodec("AAC")),
        AudioCodec::Opus => 111,
    };
    Ok(RTCRtpCodecParameters {
        capability,
        payload_type,
        ..Default::default()
    })
}

fn register_codecs(media_engine: &mut MediaEngine) -> webrtc::error::Result<()> {
    for codec in vec![RTCRtpCodecParameters {
        capability: RTCRtpCodecCapability {
            mime_type: MIME_TYPE_OPUS.to_owned(),
            clock_rate: 48000,
            channels: 2,
            sdp_fmtp_line: "minptime=10;useinbandfec=1".to_owned(),
            rtcp_feedback: vec![],
        },
        payload_type: 111,
        ..Default::default()
    }] {
        media_engine.register_codec(codec, RTPCodecType::Audio)?;
    }

    let video_rtcp_feedback = vec![
        RTCPFeedback {
            typ: "goog-remb".to_owned(),
            parameter: "".to_owned(),
        },
        RTCPFeedback {
            typ: "ccm".to_owned(),
            parameter: "fir".to_owned(),
        },
        RTCPFeedback {
            typ: "nack".to_owned(),
            parameter: "".to_owned(),
        },
        RTCPFeedback {
            typ: "nack".to_owned(),
            parameter: "pli".to_owned(),
        },
    ];
    for codec in vec![
        RTCRtpCodecParameters {
            capability: RTCRtpCodecCapability {
                mime_type: MIME_TYPE_H264.to_owned(),
                clock_rate: 90000,
                channels: 0,
                sdp_fmtp_line:
                    "level-asymmetry-allowed=1;packetization-mode=1;profile-level-id=42001f"
                        .to_owned(),
                rtcp_feedback: video_rtcp_feedback.clone(),
            },
            payload_type: 102,
            ..Default::default()
        },
        RTCRtpCodecParameters {
            capability: RTCRtpCodecCapability {
                mime_type: MIME_TYPE_H264.to_owned(),
                clock_rate: 90000,
                channels: 0,
                sdp_fmtp_line:
                    "level-asymmetry-allowed=1;packetization-mode=0;profile-level-id=42001f"
                        .to_owned(),
                rtcp_feedback: video_rtcp_feedback.clone(),
            },
            payload_type: 127,
            ..Default::default()
        },
        RTCRtpCodecParameters {
            capability: RTCRtpCodecCapability {
                mime_type: MIME_TYPE_H264.to_owned(),
                clock_rate: 90000,
                channels: 0,
                sdp_fmtp_line:
                    "level-asymmetry-allowed=1;packetization-mode=1;profile-level-id=42e01f"
                        .to_owned(),
                rtcp_feedback: video_rtcp_feedback.clone(),
            },
            payload_type: 125,
            ..Default::default()
        },
        RTCRtpCodecParameters {
            capability: RTCRtpCodecCapability {
                mime_type: MIME_TYPE_H264.to_owned(),
                clock_rate: 90000,
                channels: 0,
                sdp_fmtp_line:
                    "level-asymmetry-allowed=1;packetization-mode=0;profile-level-id=42e01f"
                        .to_owned(),
                rtcp_feedback: video_rtcp_feedback.clone(),
            },
            payload_type: 108,
            ..Default::default()
        },
        RTCRtpCodecParameters {
            capability: RTCRtpCodecCapability {
                mime_type: MIME_TYPE_H264.to_owned(),
                clock_rate: 90000,
                channels: 0,
                sdp_fmtp_line:
                    "level-asymmetry-allowed=1;packetization-mode=0;profile-level-id=42001f"
                        .to_owned(),
                rtcp_feedback: video_rtcp_feedback.clone(),
            },
            payload_type: 127,
            ..Default::default()
        },
        RTCRtpCodecParameters {
            capability: RTCRtpCodecCapability {
                mime_type: MIME_TYPE_H264.to_owned(),
                clock_rate: 90000,
                channels: 0,
                sdp_fmtp_line:
                    "level-asymmetry-allowed=1;packetization-mode=1;profile-level-id=640032"
                        .to_owned(),
                rtcp_feedback: video_rtcp_feedback.clone(),
            },
            payload_type: 123,
            ..Default::default()
        },
    ] {
        media_engine.register_codec(codec, RTPCodecType::Video)?;
    }

    Ok(())
}
