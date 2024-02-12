use compositor_pipeline::pipeline::{
    self,
    encoder::{ffmpeg_h264::Options, EncoderOptions},
    input::rtp::{AudioStream, VideoStream},
    output::rtp::RtpSenderOptions,
    OutputAudioOpts, OutputVideoOpts,
};

use self::register_request::{AudioChannels, AudioCodec, EncoderPreset, Port, VideoCodec};

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
        let input_type = pipeline::InputType {
            input_id: input_id.clone().into(),
            video: video.as_ref().map(|_video| ()),
            audio: audio.as_ref().map(|audio| pipeline::AudioOptions {
                sample_rate: audio.sample_rate,
                channels: audio.channels.clone().into(),
            }),
        };

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
            input_type,
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

        let input_type = pipeline::InputType {
            input_id: input_id.clone().into(),
            video: Some(()),
            audio: None,
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
            input_type,
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

impl From<AudioChannels> for compositor_render::AudioChannels {
    fn from(value: AudioChannels) -> Self {
        match value {
            AudioChannels::Mono => compositor_render::AudioChannels::Mono,
            AudioChannels::Stereo => compositor_render::AudioChannels::Stereo,
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
                codec: pipeline::VideoCodec::H264,
                payload_type: 96,
            }),
            audio: value.audio.map(|_| AudioStream {
                codec: pipeline::AudioCodec::Opus,
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
            Some(v) => Some(OutputVideoOpts {
                encoder_opts: EncoderOptions::H264(Options {
                    preset: v.encoder_preset.into(),
                    resolution: v.resolution.into(),
                    output_id: value.output_id.clone().into(),
                }),
                initial: v.initial.try_into()?,
            }),
            None => None,
        };

        let audio = value.audio.map(|a| OutputAudioOpts {
            initial: a.initial.into(),
            sample_rate: a.sample_rate,
            channels: a.channels.into(),
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
