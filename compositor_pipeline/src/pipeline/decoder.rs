use std::{sync::Arc, time::Duration};

use crate::{audio_mixer::InputSamples, error::DecoderInitError, queue::PipelineEvent};

use self::{
    fdk_aac::FdkAacDecoder, ffmpeg_h264::H264FfmpegDecoder, opus::OpusDecoder, resampler::Resampler,
};

use super::{
    input::ChunksReceiver,
    structs::{EncodedChunk, VideoCodec},
};

use bytes::Bytes;
use compositor_render::{Frame, InputId};
use crossbeam_channel::{bounded, unbounded, Receiver, Sender};

pub mod fdk_aac;
mod ffmpeg_h264;
mod opus;
mod resampler;

#[derive(Debug, thiserror::Error)]
pub enum ResamplerInitError {
    #[error(transparent)]
    ResamplerInitError(#[from] rubato::ResamplerConstructionError),
}

pub struct Decoder;

#[derive(Debug, Clone)]
pub struct DecoderOptions {
    pub video: Option<VideoDecoderOptions>,
    pub audio: Option<AudioDecoderOptions>,
}

impl Decoder {
    pub fn spawn(
        input_id: InputId,
        chunks: ChunksReceiver,
        decoder_options: DecoderOptions,
        output_sample_rate: u32,
    ) -> Result<DecodedDataReceiver, DecoderInitError> {
        let DecoderOptions {
            video: video_decoder_opt,
            audio: audio_decoder_opt,
        } = decoder_options;
        let ChunksReceiver {
            video: video_receiver,
            audio: audio_receiver,
        } = chunks;

        let video_receiver =
            if let (Some(opt), Some(video_receiver)) = (video_decoder_opt, video_receiver) {
                let (sender, receiver) = bounded(10);
                VideoDecoder::new(&opt, video_receiver, sender, input_id.clone())?;
                Some(receiver)
            } else {
                None
            };
        let audio_receiver =
            if let (Some(opt), Some(audio_receiver)) = (audio_decoder_opt, audio_receiver) {
                let (sender, receiver) = bounded(10);
                AudioDecoder::spawn(opt, output_sample_rate, audio_receiver, sender, input_id)?;

                Some(receiver)
            } else {
                None
            };

        Ok(DecodedDataReceiver {
            video: video_receiver,
            audio: audio_receiver,
        })
    }
}

struct AudioDecoder;

impl AudioDecoder {
    pub fn spawn(
        opts: AudioDecoderOptions,
        output_sample_rate: u32,
        chunks_receiver: Receiver<PipelineEvent<EncodedChunk>>,
        samples_sender: Sender<PipelineEvent<InputSamples>>,
        input_id: InputId,
    ) -> Result<(), DecoderInitError> {
        let (resampler_sender, resampler_receiver) = unbounded();
        let info = match opts {
            AudioDecoderOptions::Opus(opus_opt) => OpusDecoder::spawn(
                opus_opt,
                output_sample_rate,
                chunks_receiver,
                resampler_sender,
                input_id.clone(),
            )?,

            AudioDecoderOptions::Aac(aac_opt) => {
                FdkAacDecoder::spawn(aac_opt, chunks_receiver, resampler_sender, input_id.clone())?
            }
        };

        Resampler::spawn(
            input_id,
            info.decoded_sample_rate,
            output_sample_rate,
            resampler_receiver,
            samples_sender,
        )?;

        Ok(())
    }
}

enum VideoDecoder {
    H264(H264FfmpegDecoder),
}

impl VideoDecoder {
    pub fn new(
        options: &VideoDecoderOptions,
        chunks_receiver: Receiver<PipelineEvent<EncodedChunk>>,
        frame_sender: Sender<PipelineEvent<Frame>>,
        input_id: InputId,
    ) -> Result<Self, DecoderInitError> {
        match options.codec {
            VideoCodec::H264 => Ok(Self::H264(H264FfmpegDecoder::new(
                chunks_receiver,
                frame_sender,
                input_id,
            )?)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoDecoderOptions {
    pub codec: VideoCodec,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AudioDecoderOptions {
    Opus(OpusDecoderOptions),
    Aac(AacDecoderOptions),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpusDecoderOptions {
    pub forward_error_correction: bool,
}

#[derive(Debug)]
pub struct DecodedDataReceiver {
    pub video: Option<Receiver<PipelineEvent<Frame>>>,
    pub audio: Option<Receiver<PipelineEvent<InputSamples>>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AacDecoderOptions {
    pub asc: Option<Bytes>,
}

struct DecodedAudioInputInfo {
    decoded_sample_rate: u32,
}

#[derive(Debug)]
struct DecodedSamples {
    samples: Arc<Vec<(i16, i16)>>,
    start_pts: Duration,
    sample_rate: u32,
}
