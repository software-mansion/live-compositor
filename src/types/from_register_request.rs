use std::time::Duration;

use compositor_pipeline::{
    audio_mixer,
    pipeline::{
        self,
        decoder::{self, OpusDecoderOptions, VideoDecoderOptions},
        encoder::ffmpeg_h264::Options,
        input::{
            self,
            rtp::{AudioStream, VideoStream},
        },
        output::rtp::RtpSenderOptions,
    },
    queue,
};

use super::register_request::*;
use super::*;

impl TryFrom<RtpInputStream> for (compositor_render::InputId, pipeline::RegisterInputOptions) {
    type Error = TypeError;

    fn try_from(value: RtpInputStream) -> Result<Self, Self::Error> {
        let RtpInputStream {
            input_id,
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
            video: video.as_ref().map(|video| input::rtp::VideoStream {
                options: decoder::VideoDecoderOptions {
                    codec: video.clone().codec.unwrap_or(VideoCodec::H264).into(),
                },
                payload_type: video.rtp_payload_type.unwrap_or(96),
            }),
            audio: audio.as_ref().map(|audio| input::rtp::AudioStream {
                options: match audio.clone().codec.unwrap_or(AudioCodec::Opus) {
                    AudioCodec::Opus => {
                        decoder::AudioDecoderOptions::Opus(decoder::OpusDecoderOptions {
                            sample_rate: audio.sample_rate,
                            channels: audio.clone().channels.into(),
                            forward_error_correction: audio
                                .forward_error_correction
                                .unwrap_or(false),
                        })
                    }
                },
                payload_type: audio.rtp_payload_type.unwrap_or(97),
            }),
        };

        let input_options = input::InputOptions::Rtp(input::rtp::RtpReceiverOptions {
            port: port.try_into()?,
            input_id: input_id.clone().into(),
            stream: rtp_stream,
            transport_protocol: match transport_protocol.unwrap_or(TransportProtocol::Udp) {
                TransportProtocol::Udp => input::rtp::TransportProtocol::Udp,
                TransportProtocol::TcpServer => input::rtp::TransportProtocol::TcpServer,
            },
        });

        let queue_options = queue::InputOptions {
            required: required.unwrap_or(false),
            offset: offset_ms.map(|offset_ms| Duration::from_secs_f64(offset_ms / 1000.0)),
        };

        Ok((
            input_id.into(),
            pipeline::RegisterInputOptions {
                input_options,
                queue_options,
            },
        ))
    }
}

impl TryFrom<Mp4> for (compositor_render::InputId, pipeline::RegisterInputOptions) {
    type Error = TypeError;

    fn try_from(value: Mp4) -> Result<Self, Self::Error> {
        let Mp4 {
            input_id,
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

        Ok((
            input_id.clone().into(),
            pipeline::RegisterInputOptions {
                input_options: input::InputOptions::Mp4(input::mp4::Mp4Options {
                    input_id: input_id.into(),
                    source,
                }),
                queue_options,
            },
        ))
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

impl From<AudioChannels> for audio_mixer::types::AudioChannels {
    fn from(value: AudioChannels) -> Self {
        match value {
            AudioChannels::Mono => audio_mixer::types::AudioChannels::Mono,
            AudioChannels::Stereo => audio_mixer::types::AudioChannels::Stereo,
        }
    }
}

impl From<RegisterOutputRequest> for pipeline::output::OutputOptions {
    fn from(value: RegisterOutputRequest) -> Self {
        pipeline::output::OutputOptions::Rtp(RtpSenderOptions {
            port: value.port,
            ip: value.ip,
            output_id: value.output_id.into(),
            video: value.video.map(|_| VideoStream {
                options: VideoDecoderOptions {
                    codec: pipeline::VideoCodec::H264,
                },
                payload_type: 96,
            }),
            audio: value.audio.map(|a| AudioStream {
                options: decoder::AudioDecoderOptions::Opus(OpusDecoderOptions {
                    sample_rate: a.sample_rate,
                    channels: a.channels.into(),
                    forward_error_correction: a.forward_error_correction.unwrap_or(false),
                }),
                payload_type: 97,
            }),
        })
    }
}

impl TryFrom<RegisterOutputRequest> for pipeline::RegisterOutputOptions {
    type Error = TypeError;

    fn try_from(value: RegisterOutputRequest) -> Result<Self, Self::Error> {
        const NO_VIDEO_OR_AUDIO: &str =
            "At least one of \"video\" and \"audio\" fields have to be specified.";

        if value.video.is_none() && value.audio.is_none() {
            return Err(TypeError::new(NO_VIDEO_OR_AUDIO));
        }
        let output_options = value.clone().into();
        let video = match value.video {
            Some(v) => Some(pipeline::OutputVideoOptions {
                initial: v.initial.try_into()?,
                encoder_opts: pipeline::encoder::EncoderOptions::H264(Options {
                    preset: v.encoder_preset.into(),
                    resolution: v.resolution.into(),
                    output_id: value.output_id.clone().into(),
                }),
            }),
            None => None,
        };

        let audio = value.audio.map(|a| pipeline::OutputAudioOptions {
            initial: a.initial.into(),
            sample_rate: a.sample_rate,
            channels: a.channels.into(),
            forward_error_correction: a.forward_error_correction.unwrap_or(false),
        });

        Ok(Self {
            output_id: value.output_id.into(),
            output_options,
            video,
            audio,
        })
    }
}

impl From<EncoderPreset> for pipeline::encoder::ffmpeg_h264::EncoderPreset {
    fn from(value: EncoderPreset) -> Self {
        match value {
            EncoderPreset::Ultrafast => pipeline::encoder::ffmpeg_h264::EncoderPreset::Ultrafast,
            EncoderPreset::Superfast => pipeline::encoder::ffmpeg_h264::EncoderPreset::Superfast,
            EncoderPreset::Veryfast => pipeline::encoder::ffmpeg_h264::EncoderPreset::Veryfast,
            EncoderPreset::Faster => pipeline::encoder::ffmpeg_h264::EncoderPreset::Faster,
            EncoderPreset::Fast => pipeline::encoder::ffmpeg_h264::EncoderPreset::Fast,
            EncoderPreset::Medium => pipeline::encoder::ffmpeg_h264::EncoderPreset::Medium,
            EncoderPreset::Slow => pipeline::encoder::ffmpeg_h264::EncoderPreset::Slow,
            EncoderPreset::Slower => pipeline::encoder::ffmpeg_h264::EncoderPreset::Slower,
            EncoderPreset::Veryslow => pipeline::encoder::ffmpeg_h264::EncoderPreset::Veryslow,
            EncoderPreset::Placebo => pipeline::encoder::ffmpeg_h264::EncoderPreset::Placebo,
        }
    }
}
