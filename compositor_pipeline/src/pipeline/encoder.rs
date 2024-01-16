use compositor_render::{Frame, Resolution};

use crate::error::EncoderInitError;

use self::ffmpeg_h264::LibavH264Encoder;

use super::structs::EncodedChunk;

pub mod ffmpeg_h264;

pub enum Encoder {
    H264(LibavH264Encoder),
}

pub enum EncoderOptions {
    H264(ffmpeg_h264::Options),
}

impl Encoder {
    pub fn new(
        options: EncoderOptions,
    ) -> Result<(Self, Box<dyn Iterator<Item = EncodedChunk> + Send>), EncoderInitError> {
        match options {
            EncoderOptions::H264(options) => {
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
