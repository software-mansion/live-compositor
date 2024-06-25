use std::path::Path;

use crate::{error::InputInitError, queue::PipelineEvent};

use compositor_render::{Frame, InputId};
use crossbeam_channel::{bounded, Receiver};
use rtp::{RtpReceiver, RtpReceiverOptions};

use self::mp4::{Mp4, Mp4Options};

use super::{
    decoder::{
        start_audio_decoder_thread, start_audio_resampler_only_thread, start_video_decoder_thread,
        AudioDecoderOptions, DecodedDataReceiver, VideoDecoderOptions,
    },
    structs::{DecodedSamples, EncodedChunk},
    Port,
};

#[cfg(feature = "decklink")]
pub mod decklink;
pub mod mp4;
pub mod rtp;

pub enum Input {
    Rtp(RtpReceiver),
    Mp4(Mp4),
    #[cfg(feature = "decklink")]
    DeckLink(decklink::DeckLink),
}

/// Start entire processing pipeline for an input, including decoders and resamplers.
pub(super) fn start_input_threads(
    input_id: &InputId,
    options: InputOptions,
    download_dir: &Path,
    output_sample_rate: u32,
) -> Result<InputStartResult, InputInitError> {
    let InputInitResult {
        input,
        video,
        audio,
        init_info,
    } = match options {
        InputOptions::Rtp(opts) => RtpReceiver::start_new_input(input_id, opts)?,
        InputOptions::Mp4(opts) => Mp4::start_new_input(input_id, opts, download_dir)?,
        #[cfg(feature = "decklink")]
        InputOptions::DeckLink(opts) => decklink::DeckLink::start_new_input(input_id, opts)?,
    };

    let video = if let Some(video) = video {
        match video {
            VideoInputReceiver::Raw { frame_receiver } => Some(frame_receiver),
            VideoInputReceiver::Encoded {
                chunk_receiver,
                decoder_options,
            } => {
                let (sender, receiver) = bounded(10);
                start_video_decoder_thread(
                    decoder_options,
                    chunk_receiver,
                    sender,
                    input_id.clone(),
                )?;
                Some(receiver)
            }
        }
    } else {
        None
    };

    let audio = if let Some(audio) = audio {
        match audio {
            AudioInputReceiver::Raw {
                sample_receiver,
                sample_rate,
            } => {
                let (sender, receiver) = bounded(10);
                start_audio_resampler_only_thread(
                    sample_rate,
                    output_sample_rate,
                    sample_receiver,
                    sender,
                    input_id.clone(),
                )?;
                Some(receiver)
            }
            AudioInputReceiver::Encoded {
                chunk_receiver,
                decoder_options,
            } => {
                let (sender, receiver) = bounded(10);
                start_audio_decoder_thread(
                    decoder_options,
                    output_sample_rate,
                    chunk_receiver,
                    sender,
                    input_id.clone(),
                )?;
                Some(receiver)
            }
        }
    } else {
        None
    };
    Ok(InputStartResult {
        input,
        receiver: DecodedDataReceiver { video, audio },
        init_info,
    })
}

pub enum InputOptions {
    Rtp(RtpReceiverOptions),
    Mp4(Mp4Options),
    #[cfg(feature = "decklink")]
    DeckLink(decklink::DeckLinkOptions),
}

pub struct InputInitInfo {
    pub port: Option<Port>,
}

struct InputInitResult {
    input: Input,
    video: Option<VideoInputReceiver>,
    audio: Option<AudioInputReceiver>,
    init_info: InputInitInfo,
}

pub(super) struct InputStartResult {
    pub(super) input: Input,
    pub(super) receiver: DecodedDataReceiver,
    pub(super) init_info: InputInitInfo,
}

pub(super) enum VideoInputReceiver {
    #[allow(dead_code)]
    Raw {
        frame_receiver: Receiver<PipelineEvent<Frame>>,
    },
    Encoded {
        chunk_receiver: Receiver<PipelineEvent<EncodedChunk>>,
        decoder_options: VideoDecoderOptions,
    },
}

pub(super) enum AudioInputReceiver {
    #[allow(dead_code)]
    Raw {
        sample_receiver: Receiver<PipelineEvent<DecodedSamples>>,
        sample_rate: u32,
    },
    Encoded {
        chunk_receiver: Receiver<PipelineEvent<EncodedChunk>>,
        decoder_options: AudioDecoderOptions,
    },
}
