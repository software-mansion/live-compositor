use compositor_render::{Frame, Resolution};
use crossbeam_channel::{bounded, Sender};
use log::error;

use crate::{audio_mixer::types::AudioSamplesBatch, error::EncoderInitError};

use self::{ffmpeg_h264::LibavH264Encoder, opus_encoder::OpusEncoder};

use super::structs::EncodedChunk;

pub mod ffmpeg_h264;
pub mod opus_encoder;

pub struct EncoderOptions {
    pub video: Option<VideoEncoderOptions>,
    pub audio: Option<AudioEncoderOptions>,
}

#[derive(Debug, Clone)]
pub enum VideoEncoderOptions {
    H264(ffmpeg_h264::Options),
}

#[derive(Debug, Clone)]
pub enum AudioEncoderOptions {
    Opus(opus_encoder::Options),
}

#[derive(Debug, Clone)]
pub enum AudioEncoderPreset {
    Quality,
    Voip,
    LowestLatency,
}

pub struct Encoder {
    pub video: Option<VideoEncoder>,
    audio: Option<AudioEncoder>,
}

pub enum VideoEncoder {
    H264(LibavH264Encoder),
}

pub enum AudioEncoder {
    Opus(OpusEncoder),
}

impl Encoder {
    pub fn new(
        options: EncoderOptions,
        sample_rate: u32,
    ) -> Result<(Self, Box<dyn Iterator<Item = EncodedChunk> + Send>), EncoderInitError> {
        let (encoded_chunks_sender, encoded_chunks_receiver) = bounded(1);

        let video_encoder = match options.video {
            Some(video_encoder_options) => Some(VideoEncoder::new(
                video_encoder_options,
                encoded_chunks_sender.clone(),
            )?),
            None => None,
        };

        let audio_encoder = match options.audio {
            Some(audio_encoder_options) => Some(AudioEncoder::new(
                audio_encoder_options,
                sample_rate,
                encoded_chunks_sender,
            )?),
            None => None,
        };

        Ok((
            Self {
                video: video_encoder,
                audio: audio_encoder,
            },
            Box::new(encoded_chunks_receiver.into_iter()),
        ))
    }

    pub fn send_frame(&self, frame: Frame) {
        match &self.video {
            Some(video_encoder) => video_encoder.send_frame(frame),
            None => error!("Non video encoder received frame to send."),
        }
    }

    pub fn send_samples_batch(&self, batch: AudioSamplesBatch) {
        match &self.audio {
            Some(audio_encoder) => audio_encoder.send_samples_batch(batch),
            None => error!("Non audio encoder received samples to send."),
        }
    }
}

impl VideoEncoder {
    pub fn new(
        options: VideoEncoderOptions,
        sender: Sender<EncodedChunk>,
    ) -> Result<Self, EncoderInitError> {
        match options {
            VideoEncoderOptions::H264(options) => {
                Ok(Self::H264(LibavH264Encoder::new(options, sender)?))
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

impl AudioEncoder {
    fn new(
        options: AudioEncoderOptions,
        sample_rate: u32,
        sender: Sender<EncodedChunk>,
    ) -> Result<Self, EncoderInitError> {
        match options {
            AudioEncoderOptions::Opus(opus_encoder_options) => {
                OpusEncoder::new(opus_encoder_options, sample_rate, sender).map(AudioEncoder::Opus)
            }
        }
    }

    pub fn send_samples_batch(&self, batch: AudioSamplesBatch) {
        match self {
            AudioEncoder::Opus(opus_encoder) => opus_encoder.send_samples_batch(batch),
        }
    }
}
