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
    Input(RegisterInputRequest),
    Output(RegisterOutputRequest),
    Shader(ShaderSpec),
    WebRenderer(WebRendererSpec),
    Image(ImageSpec),
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct RegisterInputRequest {
    pub input_id: InputId,
    pub port: Port,
    pub video: InputVideoOptions,
    pub audio: InputAudioOptions,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct InputVideoOptions {
    codec: VideoCodec,
    rtp_payload_type: Option<u32>,
    /// Default 90_000
    clock_rate: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct InputAudioOptions {
    codec: AudioCodec,
    rtp_payload_type: Option<u32>,
    /// Default 48_000
    clock_rate: Option<u32>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VideoCodec {
    #[default]
    H264,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AudioCodec {
    #[default]
    Aac,
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
    pub audio: OutputAudioOptions,
    pub video: OutputVideoOptions,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct OutputAudioOptions {
    sample_rate: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct OutputVideoOptions {
    pub resolution: Resolution,
    pub encoder_settings: EncoderSettings,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct RegisterOutputAudioRequest {
    pub output_id: OutputId,
    pub port: u16,
    pub ip: Arc<str>,
    // At this point I'm not sure what params should be set here.
    // I think that it's implementation dependent and should be decided later on
    pub sample_rate: u32,
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

impl From<EncoderSettings> for encoder::EncoderSettings {
    fn from(settings: EncoderSettings) -> Self {
        let preset = match settings.preset.unwrap_or(EncoderPreset::Medium) {
            EncoderPreset::Ultrafast => encoder::EncoderPreset::Ultrafast,
            EncoderPreset::Superfast => encoder::EncoderPreset::Superfast,
            EncoderPreset::Veryfast => encoder::EncoderPreset::Veryfast,
            EncoderPreset::Faster => encoder::EncoderPreset::Faster,
            EncoderPreset::Fast => encoder::EncoderPreset::Fast,
            EncoderPreset::Medium => encoder::EncoderPreset::Medium,
            EncoderPreset::Slow => encoder::EncoderPreset::Slow,
            EncoderPreset::Slower => encoder::EncoderPreset::Slower,
            EncoderPreset::Veryslow => encoder::EncoderPreset::Veryslow,
            EncoderPreset::Placebo => encoder::EncoderPreset::Placebo,
        };
        Self { preset }
    }
}
