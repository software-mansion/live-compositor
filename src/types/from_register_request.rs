use compositor_pipeline::pipeline;

use self::register_request::{AudioChannels, AudioCodec, Port, VideoCodec};

use super::*;

impl TryFrom<register_request::RtpInputStream> for pipeline::RegisterInputOptions {
    type Error = TypeError;

    fn try_from(value: register_request::RtpInputStream) -> Result<Self, Self::Error> {
        let register_request::RtpInputStream {
            input_id,
            port,
            video,
            audio,
        } = value;

        const NO_VIDEO_AUDIO_SPEC: &str =
            "At least one of `video` or `audio` has to be specified in `register_input` request.";

        if video.is_none() && audio.is_none() {
            return Err(TypeError::new(NO_VIDEO_AUDIO_SPEC));
        }

        let rtp_stream = pipeline::input::rtp::RtpStream {
            video: video
                .as_ref()
                .map(|video| pipeline::input::rtp::VideoStream {
                    codec: video.codec.clone().unwrap_or(VideoCodec::H264).into(),
                    payload_type: video.rtp_payload_type.unwrap_or(96),
                }),
            audio: audio
                .as_ref()
                .map(|audio| pipeline::input::rtp::AudioStream {
                    codec: audio.codec.clone().unwrap_or(AudioCodec::Opus).into(),
                    payload_type: audio.rtp_payload_type.unwrap_or(97),
                }),
        };

        let input_options =
            pipeline::input::InputOptions::Rtp(pipeline::input::rtp::RtpReceiverOptions {
                port: port.try_into()?,
                input_id: input_id.clone().into(),
                stream: rtp_stream,
            });

        let decoder_options = pipeline::decoder::DecoderOptions {
            video: video.map(|video| pipeline::decoder::VideoDecoderOptions {
                codec: video.codec.unwrap_or(VideoCodec::H264).into(),
            }),
            audio: audio.map(|audio| match audio.codec.unwrap_or(AudioCodec::Opus) {
                AudioCodec::Opus => pipeline::decoder::AudioDecoderOptions::Opus(
                    pipeline::decoder::OpusDecoderOptions {
                        sample_rate: audio.sample_rate,
                        channels: audio.channels.into(),
                        forward_error_correction: audio.forward_error_correction.unwrap_or(false),
                    },
                ),
            }),
        };

        Ok(pipeline::RegisterInputOptions {
            input_id: input_id.into(),
            input_options,
            decoder_options,
        })
    }
}

impl TryFrom<register_request::Mp4> for pipeline::RegisterInputOptions {
    type Error = TypeError;

    fn try_from(value: register_request::Mp4) -> Result<Self, Self::Error> {
        let register_request::Mp4 {
            input_id,
            url,
            path,
        } = value;

        const BAD_URL_PATH_SPEC: &str =
            "Exactly one of `url` or `path` has to be specified in a register request for an mp4 input.";

        let source = match (url, path) {
            (Some(_), Some(_)) | (None, None) => {
                return Err(TypeError::new(BAD_URL_PATH_SPEC));
            }

            (Some(url), None) => pipeline::input::mp4::Source::Url(url),
            (None, Some(path)) => pipeline::input::mp4::Source::File(path.into()),
        };

        Ok(pipeline::RegisterInputOptions {
            input_id: input_id.clone().into(),
            input_options: pipeline::input::InputOptions::Mp4(pipeline::input::mp4::Mp4Options {
                input_id: input_id.into(),
                source,
            }),
            decoder_options: pipeline::decoder::DecoderOptions {
                video: Some(pipeline::decoder::VideoDecoderOptions {
                    codec: pipeline::VideoCodec::H264,
                }),
                audio: None,
            },
        })
    }
}

impl TryFrom<Port> for pipeline::RequestedPort {
    type Error = TypeError;

    fn try_from(value: Port) -> Result<Self, Self::Error> {
        const PORT_CONVERSION_ERROR_MESSAGE: &str = "Port needs to be a number between 1 and 65535 or a string in the \"START:END\" format, where START and END represent a range of ports.";
        match value {
            Port::U16(0) => Err(TypeError::new(PORT_CONVERSION_ERROR_MESSAGE)),
            Port::U16(v) => Ok(pipeline::RequestedPort::Exact(v)),
            Port::String(s) => {
                let (start, end) = s
                    .split_once(':')
                    .ok_or(TypeError::new(PORT_CONVERSION_ERROR_MESSAGE))?;

                let start = start
                    .parse::<u16>()
                    .or(Err(TypeError::new(PORT_CONVERSION_ERROR_MESSAGE)))?;
                let end = end
                    .parse::<u16>()
                    .or(Err(TypeError::new(PORT_CONVERSION_ERROR_MESSAGE)))?;

                if start > end {
                    return Err(TypeError::new(PORT_CONVERSION_ERROR_MESSAGE));
                }

                if start == 0 || end == 0 {
                    return Err(TypeError::new(PORT_CONVERSION_ERROR_MESSAGE));
                }

                Ok(pipeline::RequestedPort::Range((start, end)))
            }
        }
    }
}

impl From<VideoCodec> for pipeline::VideoCodec {
    fn from(value: VideoCodec) -> Self {
        match value {
            VideoCodec::H264 => pipeline::VideoCodec::H264,
        }
    }
}

impl From<AudioCodec> for pipeline::AudioCodec {
    fn from(value: AudioCodec) -> Self {
        match value {
            AudioCodec::Opus => pipeline::AudioCodec::Opus,
        }
    }
}

impl From<AudioChannels> for pipeline::AudioChannels {
    fn from(value: AudioChannels) -> Self {
        match value {
            AudioChannels::Mono => pipeline::AudioChannels::Mono,
            AudioChannels::Stereo => pipeline::AudioChannels::Stereo,
        }
    }
}

impl From<RegisterOutputRequest> for pipeline::output::OutputOptions {
    fn from(value: RegisterOutputRequest) -> Self {
        pipeline::output::OutputOptions::Rtp(pipeline::output::rtp::RtpSenderOptions {
            codec: compositor_pipeline::pipeline::VideoCodec::H264,
            ip: value.ip,
            port: value.port,
            output_id: value.output_id.into(),
        })
    }
}
