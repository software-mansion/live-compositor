use std::sync::Arc;

use compositor_pipeline::pipeline::encoder;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::renderer::*;
use super::util::*;
use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(tag = "entity_type", rename_all = "snake_case")]
pub enum RegisterRequest {
    InputStream(RegisterInputRequest),
    OutputStream(RegisterOutputRequest),
    Shader(ShaderSpec),
    WebRenderer(WebRendererSpec),
    Image(ImageSpec),
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
/// At least one of the `video` or `audio` fields have to be specified.
pub struct RegisterInputRequest {
    pub input_id: InputId,
    pub port: Port,
    pub video: Option<Video>,
    pub audio: Option<Audio>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
/// Represents video input.
pub struct Video {
    /// Default h264
    pub codec: Option<VideoCodec>,
    /// Default 96
    pub rtp_payload_type: Option<u8>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VideoCodec {
    H264,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
/// Represents audio input. 
pub struct Audio {
    /// Default opus
    pub codec: Option<AudioCodec>,
    /// Sample rate of input audio
    pub sample_rate: u32,
    pub channels: AudioChannels,
    /// Default 97
    pub rtp_payload_type: Option<u8>,
    /// Default false
    pub forward_error_correction: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AudioChannels {
    Mono,
    Stereo,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AudioCodec {
    #[default]
    Opus,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema, PartialEq, Eq)]
#[serde(untagged)]
pub enum Port {
    String(String),
    U16(u16),
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct RegisterOutputRequest {
    pub output_id: OutputId,
    pub port: u16,
    pub ip: Arc<str>,
    pub resolution: Resolution,
    pub encoder_settings: EncoderSettings,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct EncoderSettings {
    preset: Option<EncoderPreset>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EncoderPreset {
    Ultrafast,
    Superfast,
    Veryfast,
    Faster,
    Fast,
    Medium,
    Slow,
    Slower,
    Veryslow,
    Placebo,
}

impl From<RegisterOutputRequest> for encoder::EncoderOptions {
    fn from(request: RegisterOutputRequest) -> Self {
        let preset = match request
            .encoder_settings
            .preset
            .unwrap_or(EncoderPreset::Medium)
        {
            EncoderPreset::Ultrafast => encoder::ffmpeg_h264::EncoderPreset::Ultrafast,
            EncoderPreset::Superfast => encoder::ffmpeg_h264::EncoderPreset::Superfast,
            EncoderPreset::Veryfast => encoder::ffmpeg_h264::EncoderPreset::Veryfast,
            EncoderPreset::Faster => encoder::ffmpeg_h264::EncoderPreset::Faster,
            EncoderPreset::Fast => encoder::ffmpeg_h264::EncoderPreset::Fast,
            EncoderPreset::Medium => encoder::ffmpeg_h264::EncoderPreset::Medium,
            EncoderPreset::Slow => encoder::ffmpeg_h264::EncoderPreset::Slow,
            EncoderPreset::Slower => encoder::ffmpeg_h264::EncoderPreset::Slower,
            EncoderPreset::Veryslow => encoder::ffmpeg_h264::EncoderPreset::Veryslow,
            EncoderPreset::Placebo => encoder::ffmpeg_h264::EncoderPreset::Placebo,
        };
        Self::H264(encoder::ffmpeg_h264::Options {
            preset,
            resolution: request.resolution.into(),
            output_id: request.output_id.into(),
        })
    }
}
