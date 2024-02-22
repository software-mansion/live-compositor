use compositor_render::{Frame, Resolution};

use crate::error::EncoderInitError;

use self::ffmpeg_h264::LibavH264Encoder;

use super::structs::EncodedChunk;

pub mod ffmpeg_h264;
pub mod opus_encoder;

pub enum VideoEncoder {
    H264(LibavH264Encoder),
}

#[derive(Debug, Clone)]
pub enum VideoEncoderOptions {
    H264(ffmpeg_h264::Options),
}

#[derive(Debug, Clone)]
pub enum AudioEncoderOptions {
    Opus(opus_encoder::Options),
}

pub struct EncoderOptions {
    pub video: Option<VideoEncoderOptions>,
    pub audio: Option<AudioEncoderOptions>,
}

impl VideoEncoder {
    pub fn new(
        options: VideoEncoderOptions,
    ) -> Result<(Self, Box<dyn Iterator<Item = EncodedChunk> + Send>), EncoderInitError> {
        match options {
            VideoEncoderOptions::H264(options) => {
                let (encoder, iter) = LibavH264Encoder::new(options)?;
                Ok((Self::H264(encoder), iter))
            }
        }
    }

    pub fn resolution(&self) -> Resolution {
        match self {
            Self::H264(encoder) => encoder.resolution(),
        }
    }

    pub fn send_frame(&self, frame: Frame) {
        match self {
            Self::H264(encoder) => encoder.send_frame(frame),
        }
    }
}
