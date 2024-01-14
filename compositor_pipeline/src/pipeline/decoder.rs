use std::sync::Arc;

use crate::{error::DecoderInitError, queue::Queue};

use self::{ffmpeg_h264::H264FfmpegDecoder, opus_decoder::OpusDecoder};

use super::{
    input::rtp::ChunksReceiver,
    structs::{AudioChannels, AudioCodec, EncodedChunk, VideoCodec},
};
use compositor_render::InputId;
use crossbeam_channel::Receiver;

pub mod ffmpeg_h264;
mod opus_decoder;

pub enum Decoder {
    Video(VideoDecoder),
    Audio(AudioDecoder),
    VideoWithAudio {
        video: VideoDecoder,
        audio: AudioDecoder,
    },
}

#[derive(Debug, Clone)]
pub enum DecoderOptions {
    Video(VideoDecoderOptions),
    Audio(AudioDecoderOptions),
    VideoWithAudio {
        video: VideoDecoderOptions,
        audio: AudioDecoderOptions,
    },
}

impl Decoder {
    pub fn new(
        input_id: InputId,
        queue: Arc<Queue>,
        chunk: ChunksReceiver,
        decoder_options: DecoderOptions,
    ) -> Result<Self, DecoderInitError> {
        match &decoder_options {
            DecoderOptions::Video(opts) => {
                let ChunksReceiver::Video(receiver) = chunk else {
                    return Err(DecoderInitError::InvalidDecoderOptions(chunk, decoder_options));
                };

                Ok(Decoder::Video(VideoDecoder::new(
                    opts, receiver, queue, input_id,
                )?))
            }
            DecoderOptions::Audio(opts) => {
                let ChunksReceiver::Audio(receiver) = chunk else {
                    return Err(DecoderInitError::InvalidDecoderOptions(chunk, decoder_options));
                };

                Ok(Decoder::Audio(AudioDecoder::new(
                    opts, receiver, queue, input_id,
                )?))
            }
            DecoderOptions::VideoWithAudio {
                video: video_opts,
                audio: audio_opts,
            } => {
                let ChunksReceiver::VideoWithAudio { video: video_receiver, audio: audio_receiver } = chunk else {
                    return Err(DecoderInitError::InvalidDecoderOptions(chunk, decoder_options));
                };

                Ok(Decoder::VideoWithAudio {
                    video: VideoDecoder::new(
                        video_opts,
                        video_receiver,
                        queue.clone(),
                        input_id.clone(),
                    )?,
                    audio: AudioDecoder::new(audio_opts, audio_receiver, queue, input_id)?,
                })
            }
        }
    }
}

pub enum AudioDecoder {
    Opus(OpusDecoder),
}

impl AudioDecoder {
    pub fn new(
        opts: &AudioDecoderOptions,
        chunks_receiver: Receiver<EncodedChunk>,
        queue: Arc<Queue>,
        input_id: InputId,
    ) -> Result<Self, DecoderInitError> {
        let AudioDecoderOptions {
            sample_rate,
            channels,
            codec,
        } = opts;
        match codec {
            AudioCodec::Opus => Ok(AudioDecoder::Opus(OpusDecoder::new(
                *sample_rate,
                *channels,
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
pub struct AudioDecoderOptions {
    pub sample_rate: u32,
    pub channels: AudioChannels,
    pub codec: AudioCodec,
}
