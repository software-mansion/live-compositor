use std::sync::Arc;

use compositor_render::InputId;
use crossbeam_channel::{Receiver, Sender};
use tracing::{debug, error, span, Level};

use crate::{
    audio_mixer::types::{AudioSamples, AudioSamplesBatch},
    error::DecoderInitError,
    pipeline::structs::EncodedChunk,
    queue::PipelineEvent,
};

use super::OpusDecoderOptions;

pub struct OpusDecoder;

impl OpusDecoder {
    pub fn new(
        opts: OpusDecoderOptions,
        output_sample_rate: u32,
        chunks_receiver: Receiver<PipelineEvent<EncodedChunk>>,
        sample_sender: Sender<PipelineEvent<AudioSamplesBatch>>,
        input_id: InputId,
    ) -> Result<Self, DecoderInitError> {
        let decoder = opus::Decoder::new(output_sample_rate, opus::Channels::Stereo)?;

        std::thread::Builder::new()
            .name(format!("opus decoder {}", input_id.0))
            .spawn(move || {
                let _span =
                    span!(Level::INFO, "opus decoder", input_id = input_id.to_string()).entered();
                run_decoder_thread(
                    decoder,
                    opts,
                    output_sample_rate,
                    chunks_receiver,
                    sample_sender,
                )
            })
            .unwrap();

        Ok(Self)
    }
}

fn run_decoder_thread(
    mut decoder: opus::Decoder,
    opts: OpusDecoderOptions,
    output_sample_rate: u32,
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
        let decoded_samples_count =
            match decoder.decode(&chunk.data, &mut buffer, opts.forward_error_correction) {
                Ok(samples_count) => samples_count,
                Err(err) => {
                    error!("Failed to decode opus packet: {}", err);
                    continue;
                }
            };

        let mut decoded_samples = Vec::with_capacity(decoded_samples_count / 2);
        for i in 0..decoded_samples_count {
            decoded_samples.push((buffer[2 * i], buffer[2 * i + 1]));
        }

        let samples = AudioSamples::Stereo(decoded_samples);

        let samples_batch = AudioSamplesBatch {
            samples: Arc::new(samples),
            start_pts: chunk.pts,
            sample_rate: output_sample_rate,
        };

        if sample_sender
            .send(PipelineEvent::Data(samples_batch))
            .is_err()
        {
            return;
        };
    }
    if sample_sender.send(PipelineEvent::EOS).is_err() {
        debug!("Failed to send EOS from OPUS decoder. Channel closed.")
    }
}
