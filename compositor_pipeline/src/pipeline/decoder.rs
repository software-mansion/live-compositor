use std::sync::Arc;

use crate::{error::DecoderInitError, queue::Queue};

use self::{ffmpeg_h264::H264FfmpegDecoder, opus_decoder::OpusDecoder};

use super::{
    input::ChunksReceiver,
    structs::{AudioChannels, EncodedChunk, VideoCodec},
};
use compositor_render::InputId;
use crossbeam_channel::Receiver;

mod ffmpeg_h264;
mod opus_decoder;

pub struct Decoder {
    #[allow(dead_code)]
    video: Option<VideoDecoder>,
    #[allow(dead_code)]
    audio: Option<AudioDecoder>,
}

#[derive(Debug, Clone)]
pub struct DecoderOptions {
    pub video: Option<VideoDecoderOptions>,
    pub audio: Option<AudioDecoderOptions>,
}

impl Decoder {
    pub fn new(
        input_id: InputId,
        queue: Arc<Queue>,
        chunks: ChunksReceiver,
        decoder_options: DecoderOptions,
    ) -> Result<Self, DecoderInitError> {
        let DecoderOptions {
            video: video_decoder_opt,
            audio: audio_decoder_opt,
        } = decoder_options;
        let ChunksReceiver {
            video: video_receiver,
            audio: audio_receiver,
        } = chunks;

        let video_decoder =
            if let (Some(opt), Some(video_receiver)) = (video_decoder_opt, video_receiver) {
                Some(VideoDecoder::new(
                    &opt,
                    video_receiver,
                    queue.clone(),
                    input_id.clone(),
                )?)
            } else {
                None
            };
        let audio_decoder =
            if let (Some(opt), Some(audio_receiver)) = (audio_decoder_opt, audio_receiver) {
                Some(AudioDecoder::new(opt, audio_receiver, queue, input_id)?)
            } else {
                None
            };

        Ok(Self {
            video: video_decoder,
            audio: audio_decoder,
        })
    }
}

pub enum AudioDecoder {
    Opus(OpusDecoder),
}

impl AudioDecoder {
    pub fn new(
        opts: AudioDecoderOptions,
        chunks_receiver: Receiver<EncodedChunk>,
        queue: Arc<Queue>,
        input_id: InputId,
    ) -> Result<Self, DecoderInitError> {
        match opts {
            AudioDecoderOptions::Opus(opus_opt) => Ok(AudioDecoder::Opus(OpusDecoder::new(
                opus_opt,
                chunks_receiver,
                queue,
                input_id,
            )?)),
        }
    }
}

pub enum VideoDecoder {
    H264(H264FfmpegDecoder),
}

impl VideoDecoder {
    pub fn new(
        options: &VideoDecoderOptions,
        chunks_receiver: Receiver<EncodedChunk>,
        queue: Arc<Queue>,
        input_id: InputId,
    ) -> Result<Self, DecoderInitError> {
        match options.codec {
            VideoCodec::H264 => Ok(Self::H264(H264FfmpegDecoder::new(
                chunks_receiver,
                queue,
                input_id,
            )?)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct VideoDecoderOptions {
    pub codec: VideoCodec,
}

#[derive(Debug, Clone)]
pub enum AudioDecoderOptions {
    Opus(OpusDecoderOptions),
}

#[derive(Debug, Clone)]
pub struct OpusDecoderOptions {
    pub sample_rate: u32,
    pub channels: AudioChannels,
    pub forward_error_correction: bool,
}
