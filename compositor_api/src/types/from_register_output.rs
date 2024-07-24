use compositor_pipeline::pipeline::{
    self,
    encoder::{
        self,
        ffmpeg_h264::{self},
    },
    output::{
        self,
        mp4::{Mp4AudioTrack, Mp4OutputOptions, Mp4VideoTrack},
    },
};

use super::register_output::*;
use super::util::*;
use super::*;

impl TryFrom<RtpOutputStream> for pipeline::RegisterOutputOptions<output::OutputOptions> {
    type Error = TypeError;

    fn try_from(request: RtpOutputStream) -> Result<Self, Self::Error> {
        let RtpOutputStream {
            port,
            ip,
            transport_protocol,
            video,
            audio,
        } = request;
        let video_codec = video.as_ref().map(|v| match v.encoder {
            VideoEncoderOptions::FfmpegH264 { .. } => pipeline::VideoCodec::H264,
        });
        let audio_codec = audio.as_ref().map(|a| match a.encoder {
            AudioEncoderOptions::Opus { .. } => pipeline::AudioCodec::Opus,
        });

        let ConvertedOptions {
            video_encoder_options,
            video_options,
            audio_encoder_options,
            audio_options,
        } = (video, audio).try_into()?;

        let connection_options = match transport_protocol.unwrap_or(TransportProtocol::Udp) {
            TransportProtocol::Udp => {
                let pipeline::rtp::RequestedPort::Exact(port) = port.try_into()? else {
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

        let output_options = output::OutputOptions {
            output_protocol: output::OutputProtocolOptions::Rtp(output::rtp::RtpSenderOptions {
                connection_options,
                video: video_codec,
                audio: audio_codec,
            }),
            video: video_encoder_options,
            audio: audio_encoder_options,
        };

        Ok(Self {
            output_options,
            video: video_options,
            audio: audio_options,
        })
    }
}

impl TryFrom<Mp4Output> for pipeline::RegisterOutputOptions<output::OutputOptions> {
    type Error = TypeError;

    fn try_from(request: Mp4Output) -> Result<Self, Self::Error> {
        let Mp4Output { path, video, audio } = request;

        let mp4_video = video.as_ref().map(|v| match v.encoder {
            VideoEncoderOptions::FfmpegH264 { .. } => Mp4VideoTrack {
                codec: pipeline::VideoCodec::H264,
                width: v.resolution.width as u32,
                height: v.resolution.height as u32,
            },
        });
        let mp4_audio = audio.as_ref().map(|a| match a.encoder {
            AudioEncoderOptions::Opus { .. } => Mp4AudioTrack {
                codec: pipeline::AudioCodec::Opus,
            },
        });

        let ConvertedOptions {
            video_encoder_options,
            video_options,
            audio_encoder_options,
            audio_options,
        } = (video, audio).try_into()?;

        let output_options = output::OutputOptions {
            output_protocol: output::OutputProtocolOptions::Mp4(Mp4OutputOptions {
                output_path: path.into(),
                video: mp4_video,
                audio: mp4_audio,
            }),
            video: video_encoder_options,
            audio: audio_encoder_options,
        };

        Ok(Self {
            output_options,
            video: video_options,
            audio: audio_options,
        })
    }
}

struct ConvertedOptions {
    video_encoder_options: Option<pipeline::encoder::VideoEncoderOptions>,
    video_options: Option<pipeline::OutputVideoOptions>,
    audio_encoder_options: Option<pipeline::encoder::AudioEncoderOptions>,
    audio_options: Option<pipeline::OutputAudioOptions>,
}

impl TryFrom<(Option<OutputVideoOptions>, Option<OutputAudioOptions>)> for ConvertedOptions {
    type Error = TypeError;

    fn try_from(
        value: (Option<OutputVideoOptions>, Option<OutputAudioOptions>),
    ) -> Result<Self, Self::Error> {
        let (video, audio) = value;
        if video.is_none() && audio.is_none() {
            return Err(TypeError::new(
                "At least one of \"video\" and \"audio\" fields have to be specified.",
            ));
        }

        let (video_options, video_encoder_options) = match video.clone() {
            Some(v) => {
                if v.resolution.width % 2 != 0 || v.resolution.height % 2 != 0 {
                    return Err(TypeError::new(
                        "Output video width and height has to be divisible by 2",
                    ));
                };

                let VideoEncoderOptions::FfmpegH264 {
                    preset,
                    ffmpeg_options,
                } = v.encoder;

                (
                    Some(pipeline::OutputVideoOptions {
                        initial: v.initial.try_into()?,
                        end_condition: v.send_eos_when.unwrap_or_default().try_into()?,
                    }),
                    Some(pipeline::encoder::VideoEncoderOptions::H264(
                        ffmpeg_h264::Options {
                            preset: preset.into(),
                            resolution: v.resolution.into(),
                            raw_options: ffmpeg_options.unwrap_or_default().into_iter().collect(),
                        },
                    )),
                )
            }
            None => (None, None),
        };

        let (audio_options, audio_encoder_options) = match audio.clone() {
            Some(a) => {
                let AudioEncoderOptions::Opus { channels, preset } = a.encoder;

                (
                    Some(pipeline::OutputAudioOptions {
                        initial: a.initial.try_into()?,
                        channels: channels.clone().into(),
                        end_condition: a.send_eos_when.unwrap_or_default().try_into()?,
                        mixing_strategy: a
                            .mixing_strategy
                            .unwrap_or(MixingStrategy::SumClip)
                            .into(),
                    }),
                    Some(pipeline::encoder::AudioEncoderOptions::Opus(
                        encoder::opus::Options {
                            channels: channels.into(),
                            preset: preset.unwrap_or(OpusEncoderPreset::Voip).into(),
                        },
                    )),
                )
            }
            None => (None, None),
        };

        Ok(ConvertedOptions {
            video_encoder_options,
            video_options,
            audio_encoder_options,
            audio_options,
        })
    }
}

impl TryFrom<OutputEndCondition> for pipeline::PipelineOutputEndCondition {
    type Error = TypeError;

    fn try_from(value: OutputEndCondition) -> Result<Self, Self::Error> {
        match value {
            OutputEndCondition {
                any_of: Some(any_of),
                all_of: None,
                any_input: None,
                all_inputs: None,
            } => Ok(pipeline::PipelineOutputEndCondition::AnyOf(
                any_of.into_iter().map(Into::into).collect(),
            )),
            OutputEndCondition {
                any_of: None,
                all_of: Some(all_of),
                any_input: None,
                all_inputs: None,
            } => Ok(pipeline::PipelineOutputEndCondition::AllOf(
                all_of.into_iter().map(Into::into).collect(),
            )),
            OutputEndCondition {
                any_of: None,
                all_of: None,
                any_input: Some(true),
                all_inputs: None,
            } => Ok(pipeline::PipelineOutputEndCondition::AnyInput),
            OutputEndCondition {
                any_of: None,
                all_of: None,
                any_input: None,
                all_inputs: Some(true),
            } => Ok(pipeline::PipelineOutputEndCondition::AllInputs),
            OutputEndCondition {
                any_of: None,
                all_of: None,
                any_input: None | Some(false),
                all_inputs: None | Some(false),
            } => Ok(pipeline::PipelineOutputEndCondition::Never),
            _ => Err(TypeError::new(
                "Only one of \"any_of, all_of, any_input or all_inputs\" is allowed.",
            )),
        }
    }
}

impl From<H264EncoderPreset> for encoder::ffmpeg_h264::EncoderPreset {
    fn from(value: H264EncoderPreset) -> Self {
        match value {
            H264EncoderPreset::Ultrafast => ffmpeg_h264::EncoderPreset::Ultrafast,
            H264EncoderPreset::Superfast => ffmpeg_h264::EncoderPreset::Superfast,
            H264EncoderPreset::Veryfast => ffmpeg_h264::EncoderPreset::Veryfast,
            H264EncoderPreset::Faster => ffmpeg_h264::EncoderPreset::Faster,
            H264EncoderPreset::Fast => ffmpeg_h264::EncoderPreset::Fast,
            H264EncoderPreset::Medium => ffmpeg_h264::EncoderPreset::Medium,
            H264EncoderPreset::Slow => ffmpeg_h264::EncoderPreset::Slow,
            H264EncoderPreset::Slower => ffmpeg_h264::EncoderPreset::Slower,
            H264EncoderPreset::Veryslow => ffmpeg_h264::EncoderPreset::Veryslow,
            H264EncoderPreset::Placebo => ffmpeg_h264::EncoderPreset::Placebo,
        }
    }
}

impl From<OpusEncoderPreset> for encoder::AudioEncoderPreset {
    fn from(value: OpusEncoderPreset) -> Self {
        match value {
            OpusEncoderPreset::Quality => encoder::AudioEncoderPreset::Quality,
            OpusEncoderPreset::Voip => encoder::AudioEncoderPreset::Voip,
            OpusEncoderPreset::LowestLatency => encoder::AudioEncoderPreset::LowestLatency,
        }
    }
}
