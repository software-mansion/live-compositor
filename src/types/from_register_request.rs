use std::time::Duration;

use compositor_pipeline::{
    audio_mixer,
    pipeline::{
        self, decoder,
        encoder::{
            self,
            ffmpeg_h264::{self, Options},
        },
        input,
        output::{self, rtp::RtpSenderOptions},
        rtp,
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
            video: video.as_ref().map(|video| input::rtp::InputVideoStream {
                options: decoder::VideoDecoderOptions {
                    codec: video.clone().codec.unwrap_or(VideoCodec::H264).into(),
                },
            }),
            audio: audio.as_ref().map(|audio| input::rtp::InputAudioStream {
                options: match audio.clone().codec.unwrap_or(AudioCodec::Opus) {
                    AudioCodec::Opus => {
                        decoder::AudioDecoderOptions::Opus(decoder::OpusDecoderOptions {
                            forward_error_correction: audio
                                .forward_error_correction
                                .unwrap_or(false),
                        })
                    }
                },
            }),
        };

        let input_options = input::InputOptions::Rtp(input::rtp::RtpReceiverOptions {
            port: port.try_into()?,
            input_id: input_id.clone().into(),
            stream: rtp_stream,
            transport_protocol: match transport_protocol.unwrap_or(TransportProtocol::Udp) {
                TransportProtocol::Udp => rtp::TransportProtocol::Udp,
                TransportProtocol::TcpServer => rtp::TransportProtocol::TcpServer,
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

impl TryFrom<Port> for rtp::RequestedPort {
    type Error = TypeError;

    fn try_from(value: Port) -> Result<Self, Self::Error> {
        const PORT_CONVERSION_ERROR_MESSAGE: &str = "Port needs to be a number between 1 and 65535 or a string in the \"START:END\" format, where START and END represent a range of ports.";
        match value {
            Port::U16(0) => Err(TypeError::new(PORT_CONVERSION_ERROR_MESSAGE)),
            Port::U16(v) => Ok(rtp::RequestedPort::Exact(v)),
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

                Ok(rtp::RequestedPort::Range((start, end)))
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

impl TryFrom<RegisterOutputRequest> for pipeline::RegisterOutputOptions {
    type Error = TypeError;

    fn try_from(request: RegisterOutputRequest) -> Result<Self, Self::Error> {
        let RegisterOutputRequest {
            output_id,
            port,
            ip,
            transport_protocol,
            video,
            audio,
        } = request;

        if video.is_none() && audio.is_none() {
            return Err(TypeError::new(
                "At least one of \"video\" and \"audio\" fields have to be specified.",
            ));
        }

        let output_video_options = match video.clone() {
            Some(v) => {
                if v.resolution.width % 2 != 0 || v.resolution.height % 2 != 0 {
                    return Err(TypeError::new(
                        "Output video width and height has to be divisible by 2",
                    ));
                };

                Some(pipeline::OutputVideoOptions {
                    initial: v.initial.try_into()?,
                    encoder_opts: pipeline::encoder::VideoEncoderOptions::H264(Options {
                        preset: v.encoder_preset.into(),
                        resolution: v.resolution.into(),
                        output_id: output_id.clone().into(),
                    }),
                })
            }
            None => None,
        };

        let output_audio_options = match audio.clone() {
            Some(a) => Some(pipeline::OutputAudioOptions {
                initial: a.initial.try_into()?,
                channels: a.channels.into(),
                forward_error_correction: a.forward_error_correction.unwrap_or(false),
                encoder_preset: a.encoder_preset.unwrap_or(AudioEncoderPreset::Voip).into(),
            }),
            None => None,
        };

        let connection_options = match transport_protocol.unwrap_or(TransportProtocol::Udp) {
            TransportProtocol::Udp => {
                let rtp::RequestedPort::Exact(port) = port.try_into()? else {
                    return Err(TypeError::new(
                        "Port range can not be used with UDP output stream (transport_protocol=\"udp\").",
                    ));
                };
                let Some(ip) = ip else {
                    return Err(TypeError::new(
                        "\"ip\" field is required when registering output UDP stream (transport_protocol=\"udp\").",
                    ));
                };
                output::rtp::RtpConnectionOptions::Udp {
                    port: pipeline::Port(port),
                    ip,
                }
            }
            TransportProtocol::TcpServer => {
                if ip.is_some() {
                    return Err(TypeError::new(
                        "\"ip\" field is not allowed when registering TCP server connection (transport_protocol=\"tcp_server\").",
                    ));
                }

                output::rtp::RtpConnectionOptions::TcpServer {
                    port: port.try_into()?,
                }
            }
        };

        let output_options = output::OutputOptions::Rtp(RtpSenderOptions {
            output_id: output_id.clone().into(),
            connection_options,
            video: video.map(|_| pipeline::VideoCodec::H264),
            audio: audio.map(|_| pipeline::AudioCodec::Opus),
        });

        Ok(Self {
            output_id: output_id.into(),
            output_options,
            video: output_video_options,
            audio: output_audio_options,
        })
    }
}

impl From<VideoEncoderPreset> for encoder::ffmpeg_h264::EncoderPreset {
    fn from(value: VideoEncoderPreset) -> Self {
        match value {
            VideoEncoderPreset::Ultrafast => ffmpeg_h264::EncoderPreset::Ultrafast,
            VideoEncoderPreset::Superfast => ffmpeg_h264::EncoderPreset::Superfast,
            VideoEncoderPreset::Veryfast => ffmpeg_h264::EncoderPreset::Veryfast,
            VideoEncoderPreset::Faster => ffmpeg_h264::EncoderPreset::Faster,
            VideoEncoderPreset::Fast => ffmpeg_h264::EncoderPreset::Fast,
            VideoEncoderPreset::Medium => ffmpeg_h264::EncoderPreset::Medium,
            VideoEncoderPreset::Slow => ffmpeg_h264::EncoderPreset::Slow,
            VideoEncoderPreset::Slower => ffmpeg_h264::EncoderPreset::Slower,
            VideoEncoderPreset::Veryslow => ffmpeg_h264::EncoderPreset::Veryslow,
            VideoEncoderPreset::Placebo => ffmpeg_h264::EncoderPreset::Placebo,
        }
    }
}

impl From<AudioEncoderPreset> for encoder::AudioEncoderPreset {
    fn from(value: AudioEncoderPreset) -> Self {
        match value {
            AudioEncoderPreset::Quality => encoder::AudioEncoderPreset::Quality,
            AudioEncoderPreset::Voip => encoder::AudioEncoderPreset::Voip,
            AudioEncoderPreset::LowestLatency => encoder::AudioEncoderPreset::LowestLatency,
        }
    }
}
