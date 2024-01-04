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
    // Default: RTP clock rate 90_000, payload type -> first received >= 96
    pub stream: Option<Stream>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct InputAudioOptions {
    pub codec: AudioCodec,
    // Default: RTP clock rate 44_000, payload type -> first received <= 35
    pub stream: Option<Stream>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub enum Stream {
    Rtp(RtpOptions),
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct RtpOptions {
    /// If not specified, takes payload type of first input packet
    /// of range [96, 127] for video and [35, 63] for audio
    rtp_payload_type: Option<u32>,
    /// If not specified use 90_000 for video and 48_000 for audio
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
    pub audio: Option<OutputAudioOptions>,
    pub video: Option<OutputVideoOptions>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct OutputAudioOptions {
    sample_rate: u32,
    // Default aac
    codec: Option<AudioCodec>,
    // Default RTP, clock rate 44_000, rtp pty 35
    stream: Option<Stream>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct OutputVideoOptions {
    pub resolution: Resolution,
    pub encoder_settings: EncoderSettings,
    // Default h264
    codec: Option<VideoCodec>,
    // Default RTP, clock rate 90_000, rtp pty 96
    stream: Option<Stream>,
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
