use compositor_pipeline::pipeline::{
    self,
    encoder::{
        self,
        ffmpeg_h264::{self, Options},
    },
    output::{self, rtp::RtpSenderOptions},
    rtp,
};

use super::register_output::*;
use super::util::*;
use super::*;

impl TryFrom<RtpOutputStream> for pipeline::RegisterOutputOptions {
    type Error = TypeError;

    fn try_from(request: RtpOutputStream) -> Result<Self, Self::Error> {
        let RtpOutputStream {
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

                let VideoEncoderOptions::FfmpegH264 {
                    preset,
                    ffmpeg_options,
                } = v.encoder;

                Some(pipeline::OutputVideoOptions {
                    initial: v.initial.try_into()?,
                    encoder_opts: pipeline::encoder::VideoEncoderOptions::H264(Options {
                        preset: preset.into(),
                        resolution: v.resolution.into(),
                        raw_options: ffmpeg_options.unwrap_or_default().into_iter().collect(),
                    }),
                    end_condition: v.send_eos_when.unwrap_or_default().try_into()?,
                })
            }
            None => None,
        };

        let output_audio_options = match audio.clone() {
            Some(a) => {
                let AudioEncoderOptions::Opus {
                    channels,
                    preset,
                    forward_error_correction,
                } = a.encoder;

                Some(pipeline::OutputAudioOptions {
                    initial: a.initial.try_into()?,
                    channels: channels.into(),
                    forward_error_correction: forward_error_correction.unwrap_or(false),
                    encoder_preset: preset.unwrap_or(OpusEncoderPreset::Voip).into(),
                    end_condition: a.send_eos_when.unwrap_or_default().try_into()?,
                    mixing_strategy: a.mixing_strategy.unwrap_or(MixingStrategy::SumClip).into(),
                })
            }
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
            connection_options,
            video: video.map(|_| pipeline::VideoCodec::H264),
            audio: audio.map(|_| pipeline::AudioCodec::Opus),
        });

        Ok(Self {
            output_options,
            video: output_video_options,
            audio: output_audio_options,
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
