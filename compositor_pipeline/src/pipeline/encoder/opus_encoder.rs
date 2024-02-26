use std::thread::JoinHandle;

use crossbeam_channel::{bounded, Receiver, Sender};
use log::error;

use crate::{
    audio_mixer::types::{AudioChannels, AudioSamples, AudioSamplesBatch},
    error::EncoderInitError,
    pipeline::{
        structs::{EncodedChunk, EncodedChunkKind},
        AudioCodec,
    },
};

use super::AudioEncoderPreset;

#[derive(Debug, Clone)]
pub struct Options {
    pub channels: AudioChannels,
    pub preset: AudioEncoderPreset,
}

pub struct OpusEncoder {
    samples_batch_sender: Sender<Message>,
    encoder_handle: Option<JoinHandle<()>>,
}

impl OpusEncoder {
    pub fn new(
        options: Options,
        sample_rate: u32,
        packets_sender: Sender<EncodedChunk>,
    ) -> Result<Self, EncoderInitError> {
        let (init_result_sender, init_result_receiver) = bounded(0);
        let (samples_batch_sender, samples_batch_receiver) = bounded(50);

        let encoder_handle = Some(
            std::thread::Builder::new()
                .name("Opus encoder thread".to_string())
                .spawn(move || {
                    Self::encoder_thread(
                        options,
                        sample_rate,
                        samples_batch_receiver,
                        packets_sender,
                        init_result_sender,
                    )
                })
                .unwrap(),
        );

        init_result_receiver.recv().unwrap()?;

        Ok(Self {
            samples_batch_sender,
            encoder_handle,
        })
    }

    pub fn send_samples_batch(&self, batch: AudioSamplesBatch) {
        self.samples_batch_sender
            .send(Message::Batch(batch))
            .unwrap();
    }

    fn encoder_thread(
        options: Options,
        sample_rate: u32,
        samples_batch_receiver: Receiver<Message>,
        packets_sender: Sender<EncodedChunk>,
        init_result_sender: Sender<Result<(), EncoderInitError>>,
    ) {
        let mut encoder =
            match opus::Encoder::new(sample_rate, options.channels.into(), options.preset.into()) {
                Ok(encoder) => {
                    init_result_sender.send(Ok(())).unwrap();
                    encoder
                }
                Err(err) => {
                    init_result_sender
                        .send(Err(EncoderInitError::OpusError(err)))
                        .unwrap();
                    return;
                }
            };

        let mut output_buffer = [0u8; 1024 * 1024];

        let mut encode = |samples: &[i16]| match encoder.encode(samples, &mut output_buffer) {
            Ok(len) => Some(bytes::Bytes::copy_from_slice(&output_buffer[..len])),
            Err(err) => {
                error!("Opus encoding error: {}", err);
                None
            }
        };

        let mut send_batch = |batch: AudioSamplesBatch| {
            let data = match batch.samples.as_ref() {
                AudioSamples::Mono(mono_samples) => {
                    let Some(data) = encode(mono_samples) else {
                        return;
                    };
                    data
                }
                AudioSamples::Stereo(stereo_samples) => {
                    let flatten_samples: Vec<i16> =
                        stereo_samples.iter().flat_map(|(l, r)| [*l, *r]).collect();
                    let Some(data) = encode(&flatten_samples) else {
                        return;
                    };
                    data
                }
            };
            let chunk = EncodedChunk {
                data,
                pts: batch.start_pts,
                dts: None,
                kind: EncodedChunkKind::Audio(AudioCodec::Opus),
            };
            packets_sender.send(chunk).unwrap();
        };

        for msg in samples_batch_receiver {
            match msg {
                Message::Batch(batch) => send_batch(batch),
                Message::Stop => break,
            }
        }
    }
}

impl Drop for OpusEncoder {
    fn drop(&mut self) {
        self.samples_batch_sender.send(Message::Stop).unwrap();
        match self.encoder_handle.take() {
            Some(handle) => handle.join().unwrap(),
            None => error!("Opus encoder thread was already joined. This should not happen.",),
        };
    }
}

enum Message {
    Batch(AudioSamplesBatch),
    Stop,
}

impl From<AudioEncoderPreset> for opus::Application {
    fn from(value: AudioEncoderPreset) -> Self {
        match value {
            AudioEncoderPreset::Quality => opus::Application::Audio,
            AudioEncoderPreset::Voip => opus::Application::Voip,
            AudioEncoderPreset::LowestLatency => opus::Application::LowDelay,
        }
    }
}
