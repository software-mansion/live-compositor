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
    /// Offset in milliseconds relative to the pipeline start (start request). If offset is
    /// not defined then stream is synchronized based on the first frames delivery time.
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
    /// (**default=`96`**) Value of payload type field in received RTP packets.
    ///
    /// Packets with different payload type won't be treated as video and included in composing.
    /// Values should be in [0, 64] or [96, 255]. Values in range [65, 95] can't be used.
    /// For more information, see [RFC](https://datatracker.ietf.org/doc/html/rfc5761#section-4)
    /// Packets with different payload type won't be treated as video and included in composing.
    pub rtp_payload_type: Option<u8>,
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
    /// Audio channels.
    pub channels: AudioChannels,
    /// (**default=`97`**) Value of payload type field in received RTP packets.
    ///
    /// Packets with different payload type won't be treated as audio and included in mixing.
    /// Values should be in range [0, 64] or [96, 255]. Values in range [65, 95] can't be used.
    /// For more information, check out [RFC](https://datatracker.ietf.org/doc/html/rfc5761#section-4).
    pub rtp_payload_type: Option<u8>,
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
    /// TCP protocol where LiveCompositor is a server side of the connection.
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
    pub encoder_preset: EncoderPreset,
    pub initial: Component,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OutputAudioOptions {
    /// Initial audio for output.
    pub initial: Audio,
    pub sample_rate: u32,
    pub channels: AudioChannels,
    /// (**default=`false`**) Specifies whether the stream use forward error correction.
    /// It's specific for Opus codec.
    /// For more information, check out [RFC](https://datatracker.ietf.org/doc/html/rfc6716#section-2.1.7).
    pub forward_error_correction: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RegisterOutputRequest {
    pub output_id: OutputId,
    pub port: u16,
    pub ip: Arc<str>,
    pub video: Option<OutputVideoOptions>,
    pub audio: Option<OutputAudioOptions>,
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
