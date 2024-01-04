use std::sync::Arc;

use compositor_pipeline::pipeline::encoder;
use compositor_pipeline::pipeline::encoder::EncoderSettings;
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
/// At least one of video / audio options have to be provided.
pub struct RegisterInputRequest {
    pub input_id: InputId,
    pub port: Port,
    pub video: Option<InputVideoOptions>,
    pub audio: Option<InputAudioOptions>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct InputVideoOptions {
    pub codec: VideoCodec,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct InputAudioOptions {
    pub codec: AudioCodec,
    sample_rate: u32,
    rtp_clock_rate: Option<u32>,
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
    pub audio: Option<OutputAudioOptions>,
    pub video: Option<OutputVideoOptions>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct OutputAudioOptions {
    sample_rate: u32,
    // Default aac
    codec: Option<AudioCodec>,
    // default 48_000
    rtp_clock_rate: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct OutputVideoOptions {
    pub resolution: Resolution,
    #[serde(default)]
    pub encoder_preset: EncoderPreset,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EncoderPreset {
    Ultrafast,
    Superfast,
    Veryfast,
    Faster,
    Fast,
    #[default]
    Medium,
    Slow,
    Slower,
    Veryslow,
    Placebo,
}

impl From<EncoderPreset> for EncoderSettings {
    fn from(encoder_preset: EncoderPreset) -> encoder::EncoderSettings {
        let preset = match encoder_preset {
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

        encoder::EncoderSettings { preset }
    }
}
