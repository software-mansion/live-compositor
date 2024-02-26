use std::sync::Arc;

use compositor_render::InputId;
use crossbeam_channel::{Receiver, Sender};
use log::error;

use crate::{
    audio_mixer::types::{AudioChannels, AudioSamples, AudioSamplesBatch},
    error::DecoderInitError,
    pipeline::structs::EncodedChunk,
    queue::PipelineEvent,
};

use super::OpusDecoderOptions;

pub struct OpusDecoder;

impl OpusDecoder {
    pub fn new(
        opts: OpusDecoderOptions,
        chunks_receiver: Receiver<PipelineEvent<EncodedChunk>>,
        sample_sender: Sender<PipelineEvent<AudioSamplesBatch>>,
        input_id: InputId,
    ) -> Result<Self, DecoderInitError> {
        let decoder = opus::Decoder::new(opts.sample_rate, opts.channels.into())?;

        std::thread::Builder::new()
            .name(format!("opus decoder {}", input_id.0))
            .spawn(move || Self::run_decoding_thread(decoder, opts, chunks_receiver, sample_sender))
            .unwrap();

        Ok(Self)
    }

    fn run_decoding_thread(
        mut decoder: opus::Decoder,
        opts: OpusDecoderOptions,
        chunks_receiver: Receiver<PipelineEvent<EncodedChunk>>,
        sample_sender: Sender<PipelineEvent<AudioSamplesBatch>>,
    ) {
        // Max sample rate for opus is 48kHz.
        // Usually packets contain 20ms audio chunks, but for safety we use buffer
        // that can hold >1s of 48kHz stereo audio (96k samples)
        let mut buffer = [0i16; 100_000];
        for chunk in chunks_receiver {
            let chunk = match chunk {
                PipelineEvent::Data(chunk) => chunk,
                PipelineEvent::EOS => {
                    break;
                }
            };
            let decoded_samples_count = match decoder.decode(&chunk.data, &mut buffer, false) {
                Ok(samples_count) => samples_count,
                Err(err) => {
                    error!("Failed to decode opus packet: {}", err);
                    continue;
                }
            };

            let samples = match opts.channels {
                AudioChannels::Mono => {
                    let samples = buffer.iter().take(decoded_samples_count).cloned().collect();
                    AudioSamples::Mono(samples)
                }
                AudioChannels::Stereo => {
                    let mut samples = Vec::with_capacity(decoded_samples_count / 2);
                    for i in 0..decoded_samples_count {
                        samples.push((buffer[2 * i], buffer[2 * i + 1]));
                    }
                    AudioSamples::Stereo(samples)
                }
            };

            let samples = AudioSamplesBatch {
                samples: Arc::new(samples),
                start_pts: chunk.pts,
                sample_rate: opts.sample_rate,
            };

            if sample_sender.send(PipelineEvent::Data(samples)).is_err() {
                return;
            };
        }
        let _ = sample_sender.send(PipelineEvent::EOS);
    }
}
