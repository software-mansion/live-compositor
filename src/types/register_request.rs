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
/// Parameters of registered RTP input stream.
/// Before using input in video composition or output mixing,
/// input has to be firstly registered using `register_input` request.
///
/// At least one of `video` and `audio` has to be defined.
pub struct RegisterInputRequest {
    /// An identifier for the input stream.
    pub input_id: InputId,
    /// UDP port or port range on which the compositor should listen for the stream.
    pub port: Port,
    /// Parameters of a video source included in the RTP stream.
    pub video: Option<Video>,
    /// Parameters of an audio source included in the RTP stream.
    pub audio: Option<Audio>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct Video {
    /// (**default=`"h264"`**) Video codec.
    pub codec: Option<VideoCodec>,
    /// (**default=`96`**) Value of payload type field in received RTP packets.
    ///
    /// Packets with different payload type won't be treated as audio and included in composing.
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
pub struct Audio {
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
    /// (**default=`"false"`**) Specifies if received audio stream use forward error correction.
    /// Specific to Opus audio format.
    /// For more information, check out [RFC](https://datatracker.ietf.org/doc/html/rfc6716#section-2.1.7).
    pub forward_error_correction: Option<bool>,
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
pub struct RegisterOutputRequest {
    pub output_id: OutputId,
    pub port: u16,
    pub ip: Arc<str>,
    pub resolution: Resolution,
    pub encoder_preset: Option<EncoderPreset>,
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
        let preset = match request.encoder_preset.unwrap_or(EncoderPreset::Medium) {
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
