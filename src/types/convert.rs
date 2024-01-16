use compositor_pipeline::pipeline::decoder::{
    AudioDecoderOptions, OpusDecoderOptions, VideoDecoderOptions,
};
use compositor_pipeline::pipeline::input::rtp::RtpReceiverOptions;
use compositor_render::scene;

use crate::api::{self, UpdateScene};

use super::util::*;
use super::*;

impl From<ComponentId> for scene::ComponentId {
    fn from(id: ComponentId) -> Self {
        Self(id.0)
    }
}

impl From<scene::ComponentId> for ComponentId {
    fn from(id: scene::ComponentId) -> Self {
        Self(id.0)
    }
}

impl From<RendererId> for compositor_render::RendererId {
    fn from(id: RendererId) -> Self {
        Self(id.0)
    }
}

impl From<compositor_render::RendererId> for RendererId {
    fn from(id: compositor_render::RendererId) -> Self {
        Self(id.0)
    }
}

impl From<OutputId> for compositor_render::OutputId {
    fn from(id: OutputId) -> Self {
        id.0.into()
    }
}

impl From<compositor_render::OutputId> for OutputId {
    fn from(id: compositor_render::OutputId) -> Self {
        Self(id.0)
    }
}

impl From<InputId> for compositor_render::InputId {
    fn from(id: InputId) -> Self {
        id.0.into()
    }
}

impl From<compositor_render::InputId> for InputId {
    fn from(id: compositor_render::InputId) -> Self {
        Self(id.0)
    }
}

impl TryFrom<UpdateScene> for Vec<compositor_pipeline::pipeline::OutputScene> {
    type Error = TypeError;

    fn try_from(update_scene: UpdateScene) -> Result<Self, Self::Error> {
        update_scene
            .outputs
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<Vec<_>, TypeError>>()
    }
}

impl TryFrom<OutputScene> for compositor_pipeline::pipeline::OutputScene {
    type Error = TypeError;

    fn try_from(scene: OutputScene) -> Result<Self, Self::Error> {
        Ok(compositor_pipeline::pipeline::OutputScene {
            output_id: scene.output_id.into(),
            root: scene.root.try_into()?,
        })
    }
}

impl TryFrom<register_request::Port> for api::Port {
    type Error = TypeError;

    fn try_from(value: register_request::Port) -> Result<Self, Self::Error> {
        const PORT_CONVERSION_ERROR_MESSAGE: &str = "Port needs to be a number between 1 and 65535 or a string in the \"START:END\" format, where START and END represent a range of ports.";
        match value {
            Port::U16(0) => Err(TypeError::new(PORT_CONVERSION_ERROR_MESSAGE)),
            Port::U16(v) => Ok(api::Port::Exact(v)),
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

                Ok(api::Port::Range((start, end)))
            }
        }
    }
}

impl TryFrom<register_request::RegisterInputRequest>
    for compositor_pipeline::pipeline::RegisterInputOptions
{
    type Error = TypeError;

    fn try_from(value: register_request::RegisterInputRequest) -> Result<Self, Self::Error> {
        const NO_VIDEO_AUDIO_SPEC: &str =
            "At least one of `video` or `audio` has to be specified in `register_input` request.";
        if value.video.is_none() && value.audio.is_none() {
            return Err(TypeError::new(NO_VIDEO_AUDIO_SPEC));
        }
        let rtp_stream = compositor_pipeline::pipeline::input::rtp::RtpStream {
            video: value.video.as_ref().map(|video| {
                compositor_pipeline::pipeline::input::rtp::VideoStream {
                    codec: video.codec.clone().into(),
                    payload_type: video.rtp_payload_type.unwrap_or(96),
                }
            }),
            audio: value.audio.as_ref().map(|audio| {
                compositor_pipeline::pipeline::input::rtp::AudioStream {
                    codec: audio.codec.clone().into(),
                    payload_type: audio.rtp_payload_type.unwrap_or(97),
                }
            }),
        };

        let input_options =
            compositor_pipeline::pipeline::input::InputOptions::Rtp(RtpReceiverOptions {
                port: value.port.try_into()?,
                input_id: value.input_id.clone().into(),
                stream: rtp_stream,
            });

        let decoder_options = compositor_pipeline::pipeline::decoder::DecoderOptions {
            video: value.video.map(|video| VideoDecoderOptions {
                codec: video.codec.into(),
            }),
            audio: value.audio.map(|audio| match audio.codec {
                AudioCodec::Opus => AudioDecoderOptions::Opus(OpusDecoderOptions {
                    sample_rate: audio.sample_rate,
                    channels: audio.channels.into(),
                    forward_error_correction: audio.forward_error_correction.unwrap_or(false),
                }),
            }),
        };

        Ok(compositor_pipeline::pipeline::RegisterInputOptions {
            input_id: value.input_id.into(),
            input_options,
            decoder_options,
        })
    }
}

impl TryFrom<crate::types::Port> for compositor_pipeline::pipeline::Port {
    type Error = TypeError;

    fn try_from(value: crate::types::Port) -> Result<Self, Self::Error> {
        let port: api::Port = value.try_into()?;
        match port {
            api::Port::Range((lower, upper)) => {
                Ok(compositor_pipeline::pipeline::Port::Range((lower, upper)))
            }
            api::Port::Exact(port) => Ok(compositor_pipeline::pipeline::Port::Exact(port)),
        }
    }
}

impl From<crate::types::VideoCodec> for compositor_pipeline::pipeline::structs::VideoCodec {
    fn from(value: crate::types::VideoCodec) -> Self {
        match value {
            crate::types::VideoCodec::H264 => {
                compositor_pipeline::pipeline::structs::VideoCodec::H264
            }
        }
    }
}

impl From<crate::types::AudioCodec> for compositor_pipeline::pipeline::structs::AudioCodec {
    fn from(value: crate::types::AudioCodec) -> Self {
        match value {
            crate::types::AudioCodec::Opus => {
                compositor_pipeline::pipeline::structs::AudioCodec::Opus
            }
        }
    }
}

impl From<crate::types::AudioChannels> for compositor_pipeline::pipeline::structs::AudioChannels {
    fn from(value: crate::types::AudioChannels) -> Self {
        match value {
            crate::types::AudioChannels::Mono => {
                compositor_pipeline::pipeline::structs::AudioChannels::Mono
            }
            crate::types::AudioChannels::Stereo => {
                compositor_pipeline::pipeline::structs::AudioChannels::Stereo
            }
        }
    }
}
