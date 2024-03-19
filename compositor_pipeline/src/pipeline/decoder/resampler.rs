use std::sync::Arc;
use std::{time::Duration, vec};

use compositor_render::InputId;
use crossbeam_channel::{Receiver, Sender};
use log::error;
use rubato::Resampler as _;
use tracing::{span, Level};

use crate::audio_mixer::InputSamples;
use crate::queue::PipelineEvent;

use super::DecodedSamples;
use super::ResamplerInitError;

const SAMPLE_BATCH_DURATION: Duration = Duration::from_millis(20);

pub struct Resampler;

impl Resampler {
    pub fn spawn(
        input_id: InputId,
        input_sample_rate: u32,
        output_sample_rate: u32,
        decoder_receiver: Receiver<PipelineEvent<DecodedSamples>>,
        sender: Sender<PipelineEvent<InputSamples>>,
    ) -> Result<(), ResamplerInitError> {
        if input_sample_rate == output_sample_rate {
            Self::spawn_passthrough_resampler_thread(input_id, decoder_receiver, sender);
            Ok(())
        } else {
            Self::spawn_ffr_resampler_thread(
                input_id,
                input_sample_rate,
                output_sample_rate,
                decoder_receiver,
                sender,
            )
        }
    }

    fn spawn_passthrough_resampler_thread(
        input_id: InputId,
        decoder_receiver: Receiver<PipelineEvent<DecodedSamples>>,
        sender: Sender<PipelineEvent<InputSamples>>,
    ) {
        std::thread::Builder::new()
            .name(format!(
                "Passthrough resampler thread for input: {}",
                input_id
            ))
            .spawn(move || {
                let _span = span!(
                    Level::INFO,
                    "passthrough resampler",
                    input_id = input_id.to_string()
                )
                .entered();
                Self::passthrough_thread(decoder_receiver, sender);
            })
            .unwrap();
    }

    fn passthrough_thread(
        receiver: Receiver<PipelineEvent<DecodedSamples>>,
        sender: Sender<PipelineEvent<InputSamples>>,
    ) {
        for event in receiver {
            let PipelineEvent::Data(decoded_samples) = event else {
                break;
            };

            let input_samples = PipelineEvent::Data(InputSamples::new(
                decoded_samples.samples,
                decoded_samples.start_pts,
                decoded_samples.sample_rate,
            ));
            sender.send(input_samples).unwrap();
        }

        sender.send(PipelineEvent::EOS).unwrap();
    }

    fn spawn_ffr_resampler_thread(
        input_id: InputId,
        input_sample_rate: u32,
        output_sample_rate: u32,
        decoder_receiver: Receiver<PipelineEvent<DecodedSamples>>,
        sender: Sender<PipelineEvent<InputSamples>>,
    ) -> Result<(), ResamplerInitError> {
        let output_batch_count =
            (output_sample_rate as f64 * SAMPLE_BATCH_DURATION.as_secs_f64()).round() as usize;

        let resampler = rubato::FftFixedOut::<f32>::new(
            input_sample_rate as usize,
            output_sample_rate as usize,
            output_batch_count,
            1024,
            2,
        )?;

        std::thread::Builder::new()
            .name(format!("FFT resampler thread for input: {}", input_id))
            .spawn(move || {
                let _span = span!(
                    Level::INFO,
                    "fft resampler",
                    input_id = input_id.to_string()
                )
                .entered();
                Self::fft_resampler_thread(decoder_receiver, sender, resampler, output_sample_rate);
            })
            .unwrap();

        Ok(())
    }

    fn fft_resampler_thread(
        receiver: Receiver<PipelineEvent<DecodedSamples>>,
        sender: Sender<PipelineEvent<InputSamples>>,
        mut resampler: rubato::FftFixedOut<f32>,
        output_sample_rate: u32,
    ) {
        let mut input_buffer = vec![Vec::new(); 2];
        // let mut input_buffer = vec![VecDeque::new(); 2];
        let mut output_buffer = vec![vec![0.0; resampler.output_frames_max()]; 2];
        for event in receiver {
            let PipelineEvent::Data(decoded_samples) = event else {
                break
            };
            for (l, r) in decoded_samples.samples.iter().cloned() {
                input_buffer[0].push(l as f32);
                input_buffer[1].push(r as f32);
            }

            let mut resampled_samples = Vec::new();
            while resampler.input_frames_next() <= input_buffer.len() {
                match resampler.process_into_buffer(&input_buffer, &mut output_buffer, None) {
                    Ok((input_samples, output_samples)) => {
                        for i in 0..output_samples {
                            let l = clip_to_i16(output_buffer[0][i]);
                            let r = clip_to_i16(output_buffer[1][i]);
                            resampled_samples.push((l, r));
                        }
                        input_buffer[0].drain(0..input_samples);
                        input_buffer[1].drain(0..input_samples);
                    }
                    Err(err) => {
                        error!("Resampling error: {}", err)
                    }
                }
            }

            sender
                .send(PipelineEvent::Data(InputSamples::new(
                    Arc::new(resampled_samples),
                    decoded_samples.start_pts,
                    output_sample_rate,
                )))
                .unwrap();
        }

        sender.send(PipelineEvent::EOS).unwrap();
    }
}

fn clip_to_i16(val: f32) -> i16 {
    val.max(i16::MAX as f32).min(i16::MIN as f32) as i16
}
