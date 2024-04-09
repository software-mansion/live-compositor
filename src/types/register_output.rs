use std::collections::HashMap;
use std::sync::Arc;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::audio::*;
use super::util::*;
use super::video::*;
use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RtpOutputStream {
    /// Depends on the value of the `transport_protocol` field:
    ///
    ///   - `udp` - An UDP port number that RTP packets will be sent to.
    ///
    ///   - `tcp_server` - A local TCP port number or a port range that LiveCompositor will listen for incoming connections.
    pub port: Port,
    /// Only valid if `transport_protocol="udp"`. IP address where RTP packets should be sent to.
    pub ip: Option<Arc<str>>,
    /// (**default=`"udp"`**) Transport layer protocol that will be used to send RTP packets.
    pub transport_protocol: Option<TransportProtocol>,
    pub video: Option<OutputRtpVideoOptions>,
    pub audio: Option<OutputRtpAudioOptions>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OutputRtpVideoOptions {
    /// Output resolution in pixels.
    pub resolution: Resolution,
    /// Video encoder options.
    pub encoder: VideoEncoderOptions,
    /// Root of a component tree/scene that should be rendered for the output. Use [`update_output` request](../routes.md#update-output) to update this value after registration. [Learn more](../../concept/component.md).
    pub initial: Video,
    /// Defines when output stream should end if some of the input streams are finished. If output includes both audio and video streams, then EOS needs to be sent on both.
    pub send_eos_when: Option<OutputEndCondition>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OutputRtpAudioOptions {
    /// Initial audio mixer configuration for output.
    pub initial: Audio,
    /// Audio encoder options.
    pub encoder: AudioEncoderOptions,
    /// (**default="sum_clip") Specifies how audio should be mixed.
    pub mixing_strategy: Option<MixingStrategy>,
    /// Condition for termination of output stream based on the input streams states.
    pub send_eos_when: Option<OutputEndCondition>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum VideoEncoderOptions {
    #[serde(rename = "ffmpeg_h264")]
    FfmpegH264 {
        /// (**default=`"fast"`**) Preset for an encoder. See `FFmpeg` [docs](https://trac.ffmpeg.org/wiki/Encode/H.264#Preset) to learn more.
        preset: H264EncoderPreset,

        /// Raw FFmpeg encoder options. See [docs](https://ffmpeg.org/ffmpeg-codecs.html) for more.
        ffmpeg_options: Option<HashMap<String, String>>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum AudioEncoderOptions {
    Opus(OpusEncoderOptions),
    Acc(OpusEncoderOptions),
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OpusEncoderOptions {
    pub channels: AudioChannels,

    /// (**default="voip"**) Specifies preset for audio output encoder.
    pub preset: Option<OpusEncoderPreset>,

    /// (**default=`false`**) Specifies whether the stream use forward error correction.
    /// It's specific for Opus codec.
    /// For more information, check out [RFC](https://datatracker.ietf.org/doc/html/rfc6716#section-2.1.7).
    pub forward_error_correction: Option<bool>,
}

/// This type defines when end of an input stream should trigger end of the output stream. Only one of those fields can be set at the time.
///
/// Unless specified otherwise the input stream is considered finished/ended when:
///
/// - TCP connection was dropped/closed.
///
/// - RTCP Goodbye packet (`BYE`) was received.
///
/// - Mp4 track has ended.
///
/// - Input was unregistered already (or never registered).
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema, Default)]
#[serde(deny_unknown_fields)]
pub struct OutputEndCondition {
    /// Terminate output stream if any of the input streams from the list are finished.
    pub any_of: Option<Vec<InputId>>,
    /// Terminate output stream if all the input streams from the list are finished.
    pub all_of: Option<Vec<InputId>>,
    /// Terminate output stream if any of the input streams ends. This includes streams added after the output was registered. In particular, output stream will **not be** terminated if no inputs were ever connected.
    pub any_input: Option<bool>,
    /// Terminate output stream if all the input streams finish. In particular, output stream will **be** terminated if no inputs were ever connected.
    pub all_inputs: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum H264EncoderPreset {
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

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OpusEncoderPreset {
    /// Best for broadcast/high-fidelity application where the decoded audio
    /// should be as close as possible to the input.
    Quality,
    /// Best for most VoIP/videoconference applications where listening quality
    /// and intelligibility matter most.
    Voip,
    /// Only use when lowest-achievable latency is what matters most.
    LowestLatency,
}
