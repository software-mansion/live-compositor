use std::time::Duration;

use compositor_pipeline::{
    pipeline::{self, decoder, input},
    queue,
};

use super::register_input::*;
use super::util::*;

impl TryFrom<RtpInputStream> for pipeline::RegisterInputOptions {
    type Error = TypeError;

    fn try_from(value: RtpInputStream) -> Result<Self, Self::Error> {
        let RtpInputStream {
            port,
            video,
            audio,
            required,
            offset_ms,
            transport_protocol,
        } = value;

        const NO_VIDEO_AUDIO_SPEC: &str =
            "At least one of `video` and `audio` has to be specified in `register_input` request.";

        if video.is_none() && audio.is_none() {
            return Err(TypeError::new(NO_VIDEO_AUDIO_SPEC));
        }

        let rtp_stream = input::rtp::RtpStream {
            video: video.as_ref().map(|video| input::rtp::InputVideoStream {
                options: match video {
                    InputRtpVideoOptions::FfmepgH264 => decoder::VideoDecoderOptions {
                        codec: pipeline::VideoCodec::H264,
                    },
                },
            }),
            audio: audio.as_ref().map(|audio| input::rtp::InputAudioStream {
                options: match audio {
                    InputRtpAudioOptions::Opus {
                        forward_error_correction,
                    } => decoder::AudioDecoderOptions::Opus(decoder::OpusDecoderOptions {
                        forward_error_correction: forward_error_correction.unwrap_or(false),
                    }),
                },
            }),
        };

        let input_options = input::InputOptions::Rtp(input::rtp::RtpReceiverOptions {
            port: port.try_into()?,
            stream: rtp_stream,
            transport_protocol: transport_protocol.unwrap_or(TransportProtocol::Udp).into(),
        });

        let queue_options = queue::InputOptions {
            required: required.unwrap_or(false),
            offset: offset_ms.map(|offset_ms| Duration::from_secs_f64(offset_ms / 1000.0)),
        };

        Ok(pipeline::RegisterInputOptions {
            input_options,
            queue_options,
        })
    }
}

impl TryFrom<Mp4> for pipeline::RegisterInputOptions {
    type Error = TypeError;

    fn try_from(value: Mp4) -> Result<Self, Self::Error> {
        let Mp4 {
            url,
            path,
            required,
            offset_ms,
        } = value;

        const BAD_URL_PATH_SPEC: &str =
            "Exactly one of `url` or `path` has to be specified in a register request for an mp4 input.";

        let source = match (url, path) {
            (Some(_), Some(_)) | (None, None) => {
                return Err(TypeError::new(BAD_URL_PATH_SPEC));
            }

            (Some(url), None) => input::mp4::Source::Url(url),
            (None, Some(path)) => input::mp4::Source::File(path.into()),
        };

        let queue_options = queue::InputOptions {
            required: required.unwrap_or(false),
            offset: offset_ms.map(|offset_ms| Duration::from_secs_f64(offset_ms / 1000.0)),
        };

        Ok(pipeline::RegisterInputOptions {
            input_options: input::InputOptions::Mp4(input::mp4::Mp4Options { source }),
            queue_options,
        })
    }
}
