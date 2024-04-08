use core::f64;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::util::*;

/// Parameters for an input stream from RTP source.
/// At least one of `video` and `audio` has to be defined.
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RtpInputStream {
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
#[serde(tag = "decoder", rename_all = "snake_case", deny_unknown_fields)]
pub enum InputRtpAudioOptions {
    Opus {
        /// (**default=`false`**) Specifies whether the stream uses forward error correction.
        /// It's specific for Opus codec.
        /// For more information, check out [RFC](https://datatracker.ietf.org/doc/html/rfc6716#section-2.1.7).
        forward_error_correction: Option<bool>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(tag = "decoder", rename_all = "snake_case", deny_unknown_fields)]
pub enum InputRtpVideoOptions {
    #[serde(rename = "ffmpeg_h264")]
    FfmepgH264,
}
