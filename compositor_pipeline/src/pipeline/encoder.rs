use compositor_render::{Frame, Resolution};
use crossbeam_channel::{bounded, Receiver, Sender};
use log::error;

use crate::{audio_mixer::types::OutputSamples, error::EncoderInitError, queue::PipelineEvent};

use self::{ffmpeg_h264::LibavH264Encoder, opus::OpusEncoder};

use super::structs::EncoderOutputEvent;

pub mod ffmpeg_h264;
pub mod opus;

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
    Opus(opus::Options),
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
    ) -> Result<(Self, Receiver<EncoderOutputEvent>), EncoderInitError> {
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
            encoded_chunks_receiver,
        ))
    }

    pub fn frame_sender(&self) -> Option<&Sender<PipelineEvent<Frame>>> {
        match &self.video {
            Some(VideoEncoder::H264(encoder)) => Some(encoder.frame_sender()),
            None => {
                error!("Non video encoder received frame to send.");
                None
            }
        }
    }

    pub fn samples_batch_sender(&self) -> Option<&Sender<PipelineEvent<OutputSamples>>> {
        match &self.audio {
            Some(AudioEncoder::Opus(encoder)) => Some(encoder.samples_batch_sender()),
            None => {
                error!("Non audio encoder received samples to send.");
                None
            }
        }
    }
}

impl VideoEncoder {
    pub fn new(
        options: VideoEncoderOptions,
        sender: Sender<EncoderOutputEvent>,
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
}

impl AudioEncoder {
    fn new(
        options: AudioEncoderOptions,
        sample_rate: u32,
        sender: Sender<EncoderOutputEvent>,
    ) -> Result<Self, EncoderInitError> {
        match options {
            AudioEncoderOptions::Opus(opus_encoder_options) => {
                OpusEncoder::new(opus_encoder_options, sample_rate, sender).map(AudioEncoder::Opus)
            }
        }
    }
}
