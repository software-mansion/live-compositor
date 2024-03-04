use core::f64;
use std::sync::Arc;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::renderer::*;
use super::util::*;
use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(tag = "entity_type", rename_all = "snake_case")]
pub enum RegisterRequest {
    RtpInputStream(RtpInputStream),
    Mp4(Mp4),
    OutputStream(RegisterOutputRequest),
    Shader(ShaderSpec),
    WebRenderer(WebRendererSpec),
    Image(ImageSpec),
}

/// Parameters for an input stream from RTP source.
/// At least one of `video` and `audio` has to be defined.
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RtpInputStream {
    /// An identifier for the input stream.
    pub input_id: InputId,
    /// UDP port or port range on which the compositor should listen for the stream.
    pub port: Port,
    /// Transport protocol.
    pub transport_protocol: Option<TransportProtocol>,
    /// Parameters of a video source included in the RTP stream.
    pub video: Option<InputRtpVideoOptions>,
    /// Parameters of an audio source included in the RTP stream.
    pub audio: Option<InputRtpAudioOptions>,
    /// (**default=`false`**) If input is required and the stream is not delivered
    /// on time, then LiveCompositor will delay producing output frames.
    pub required: Option<bool>,
    /// Offset in milliseconds relative to the pipeline start (start request). If the offset is
    /// not defined then the stream will be synchronized based on the delivery time of the initial
    /// frames.
    pub offset_ms: Option<f64>,
}

/// Input stream from MP4 file.
/// Exactly one of `url` and `path` has to be defined.
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Mp4 {
    /// An identifier for the input stream.
    pub input_id: InputId,
    /// URL of the MP4 file.
    pub url: Option<String>,
    /// Path to the MP4 file.
    pub path: Option<String>,
    /// (**default=`false`**) If input is required and frames are not processed
    /// on time, then LiveCompositor will delay producing output frames.
    pub required: Option<bool>,
    /// Offset in milliseconds relative to the pipeline start (start request). If offset is
    /// not defined then stream is synchronized based on the first frames delivery time.
    pub offset_ms: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InputRtpVideoOptions {
    /// (**default=`"h264"`**) Video codec.
    pub codec: Option<VideoCodec>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VideoCodec {
    /// H264 video.
    H264,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InputRtpAudioOptions {
    /// (**default=`"opus"`**) Audio codec.
    pub codec: Option<AudioCodec>,
    /// Sample rate. If the specified sample rate doesn't match
    /// real sample rate, audio won't be mixed properly.
    pub sample_rate: u32,
    /// (**default=`false`**) Specifies whether the stream uses forward error correction.
    /// It's specific for Opus codec.
    /// For more information, check out [RFC](https://datatracker.ietf.org/doc/html/rfc6716#section-2.1.7).
    pub forward_error_correction: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TransportProtocol {
    /// UDP protocol.
    Udp,
    /// TCP protocol where LiveCompositor is the server side of the connection.
    TcpServer,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AudioChannels {
    /// Mono audio (single channel).
    Mono,
    /// Stereo audio (two channels).
    Stereo,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AudioCodec {
    /// Opus audio.
    Opus,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema, PartialEq, Eq)]
#[serde(untagged)]
pub enum Port {
    String(String),
    U16(u16),
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OutputVideoOptions {
    pub resolution: Resolution,
    pub encoder_preset: VideoEncoderPreset,
    pub initial: Component,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OutputAudioOptions {
    /// Initial audio for output.
    pub initial: Audio,
    pub channels: AudioChannels,
    /// (**default=`false`**) Specifies whether the stream use forward error correction.
    /// It's specific for Opus codec.
    /// For more information, check out [RFC](https://datatracker.ietf.org/doc/html/rfc6716#section-2.1.7).
    pub forward_error_correction: Option<bool>,
    /// (**default="voip"**) Specifies preset for audio output encoder.
    pub encoder_preset: Option<AudioEncoderPreset>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AudioEncoderPreset {
    /// Best for broadcast/high-fidelity application where the decoded audio
    /// should be as close as possible to the input.
    Quality,
    /// Best for most VoIP/videoconference applications where listening quality
    /// and intelligibility matter most.
    Voip,
    /// Only use when lowest-achievable latency is what matters most.
    LowestLatency,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RegisterOutputRequest {
    pub output_id: OutputId,
    pub port: Port,
    pub ip: Option<Arc<str>>,
    pub transport_protocol: Option<TransportProtocol>,
    pub video: Option<OutputVideoOptions>,
    pub audio: Option<OutputAudioOptions>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VideoEncoderPreset {
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
