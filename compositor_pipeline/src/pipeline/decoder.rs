use crate::{audio_mixer::InputSamples, error::DecoderInitError, queue::PipelineEvent};

use super::{input::ChunksReceiver, structs::VideoCodec};

use bytes::Bytes;
use compositor_render::{Frame, InputId};
use crossbeam_channel::{bounded, Receiver};

pub use audio::AacDecoderError;

mod audio;
mod video;

pub fn start_decoder(
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
            video::start_video_decoder_thread(&opt, video_receiver, sender, input_id.clone())?;
            Some(receiver)
        } else {
            None
        };
    let audio_receiver =
        if let (Some(opt), Some(audio_receiver)) = (audio_decoder_opt, audio_receiver) {
            let (sender, receiver) = bounded(10);
            audio::start_audio_decoder_thread(
                opt,
                output_sample_rate,
                audio_receiver,
                sender,
                input_id,
            )?;
            Some(receiver)
        } else {
            None
        };

    Ok(DecodedDataReceiver {
        video: video_receiver,
        audio: audio_receiver,
    })
}

#[derive(Debug, Clone)]
pub struct DecoderOptions {
    pub video: Option<VideoDecoderOptions>,
    pub audio: Option<AudioDecoderOptions>,
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
