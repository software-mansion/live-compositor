use crate::{audio_mixer::types::AudioSamplesBatch, error::DecoderInitError, queue::PipelineEvent};

use self::{fdk_aac::FdkAacDecoder, ffmpeg_h264::H264FfmpegDecoder, opus::OpusDecoder};

use super::{
    input::ChunksReceiver,
    structs::{EncodedChunk, VideoCodec},
};

use bytes::Bytes;
use compositor_render::{Frame, InputId};
use crossbeam_channel::{bounded, Receiver, Sender};

pub mod fdk_aac;
mod ffmpeg_h264;
mod opus;

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
        chunks: ChunksReceiver,
        decoder_options: DecoderOptions,
        output_sample_rate: u32,
    ) -> Result<(Self, DecodedDataReceiver), DecoderInitError> {
        let DecoderOptions {
            video: video_decoder_opt,
            audio: audio_decoder_opt,
        } = decoder_options;
        let ChunksReceiver {
            video: video_receiver,
            audio: audio_receiver,
        } = chunks;

        let (video_decoder, video_receiver) =
            if let (Some(opt), Some(video_receiver)) = (video_decoder_opt, video_receiver) {
                let (sender, receiver) = bounded(10);
                (
                    Some(VideoDecoder::new(
                        &opt,
                        video_receiver,
                        sender,
                        input_id.clone(),
                    )?),
                    Some(receiver),
                )
            } else {
                (None, None)
            };
        let (audio_decoder, audio_receiver) =
            if let (Some(opt), Some(audio_receiver)) = (audio_decoder_opt, audio_receiver) {
                let (sender, receiver) = bounded(10);
                (
                    Some(AudioDecoder::new(
                        opt,
                        output_sample_rate,
                        audio_receiver,
                        sender,
                        input_id,
                    )?),
                    Some(receiver),
                )
            } else {
                (None, None)
            };

        Ok((
            Self {
                video: video_decoder,
                audio: audio_decoder,
            },
            DecodedDataReceiver {
                video: video_receiver,
                audio: audio_receiver,
            },
        ))
    }
}

pub enum AudioDecoder {
    Opus(OpusDecoder),
    FdkAac(FdkAacDecoder),
}

impl AudioDecoder {
    pub fn new(
        opts: AudioDecoderOptions,
        output_sample_rate: u32,
        chunks_receiver: Receiver<PipelineEvent<EncodedChunk>>,
        samples_sender: Sender<PipelineEvent<AudioSamplesBatch>>,
        input_id: InputId,
    ) -> Result<Self, DecoderInitError> {
        match opts {
            AudioDecoderOptions::Opus(opus_opt) => Ok(AudioDecoder::Opus(OpusDecoder::new(
                opus_opt,
                output_sample_rate,
                chunks_receiver,
                samples_sender,
                input_id,
            )?)),

            AudioDecoderOptions::Aac(aac_opt) => Ok(AudioDecoder::FdkAac(FdkAacDecoder::new(
                aac_opt,
                chunks_receiver,
                samples_sender,
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
    pub audio: Option<Receiver<PipelineEvent<AudioSamplesBatch>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AacTransport {
    RawAac,
    ADTS,
    ADIF,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AacDecoderOptions {
    pub transport: AacTransport,
    pub asc: Option<Bytes>,
}
