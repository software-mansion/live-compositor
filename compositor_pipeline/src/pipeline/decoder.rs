use std::sync::Arc;

use crate::{error::DecoderInitError, queue::Queue};

use self::ffmpeg_h264::H264FfmpegDecoder;

use super::structs::EncodedChunk;
use compositor_render::InputId;

pub mod ffmpeg_h264;

pub enum Decoder {
    H264(H264FfmpegDecoder),
}

impl Decoder {
    pub fn new(
        parameters: DecoderOptions,
        chunks: Box<dyn Iterator<Item = EncodedChunk> + Send>,
        queue: Arc<Queue>,
        input_id: InputId,
    ) -> Result<Self, DecoderInitError> {
        match parameters {
            DecoderOptions::H264 => {
                Ok(Self::H264(H264FfmpegDecoder::new(chunks, queue, input_id)?))
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DecoderOptions {
    H264,
}
