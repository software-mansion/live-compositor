use std::sync::Arc;
use std::{time::Duration, vec};

use compositor_render::InputId;
use crossbeam_channel::{Receiver, Sender};
use log::{debug, error};
use rubato::Resampler as _;
use tracing::{span, Level};

use crate::audio_mixer::InputSamples;
use crate::queue::PipelineEvent;

use super::DecodedSamples;

const SAMPLE_BATCH_DURATION: Duration = Duration::from_millis(20);

pub struct Resampler;

impl Resampler {
    pub fn spawn(
        input_id: InputId,
        input_sample_rate: u32,
        output_sample_rate: u32,
        decoder_receiver: Receiver<PipelineEvent<DecodedSamples>>,
        sender: Sender<PipelineEvent<InputSamples>>,
    ) -> Result<(), rubato::ResamplerConstructionError> {
        if input_sample_rate == output_sample_rate {
            Self::spawn_passthrough_resampler_thread(input_id, decoder_receiver, sender);
            Ok(())
        } else {
            Self::spawn_fft_resampler_thread(
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

    fn spawn_fft_resampler_thread(
        input_id: InputId,
        input_sample_rate: u32,
        output_sample_rate: u32,
        decoder_receiver: Receiver<PipelineEvent<DecodedSamples>>,
        sender: Sender<PipelineEvent<InputSamples>>,
    ) -> Result<(), rubato::ResamplerConstructionError> {
        /// This part of pipeline use stereo
        const CHANNELS: usize = 2;
        /// Not sure what should be here, but rubato example used 2
        /// https://github.com/HEnquist/rubato/blob/master/examples/process_f64.rs#L174
        const SUB_CHUNKS: usize = 2;
        let output_batch_size =
            (output_sample_rate as f64 * SAMPLE_BATCH_DURATION.as_secs_f64()).round() as usize;

        let resampler = rubato::FftFixedOut::<f64>::new(
            input_sample_rate as usize,
            output_sample_rate as usize,
            output_batch_size,
            SUB_CHUNKS,
            CHANNELS,
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
                Self::fft_resampler_thread(
                    decoder_receiver,
                    sender,
                    resampler,
                    input_sample_rate,
                    output_sample_rate,
                );
            })
            .unwrap();

        Ok(())
    }

    fn fft_resampler_thread(
        receiver: Receiver<PipelineEvent<DecodedSamples>>,
        sender: Sender<PipelineEvent<InputSamples>>,
        mut resampler: rubato::FftFixedOut<f64>,
        input_sample_rate: u32,
        output_sample_rate: u32,
    ) {
        // input and output buffers are used to reduce allocations

        // resampler.input_buffer_allocate() use only resampler.input_frames_max() as capacity,
        // but since this code push whole input batch into this buffer and fill missing samples it
        // will perform re-allocation anyway. With resampler.input_frames_max() * 10 re-allocation
        // probably won't happen (only in case of missing many consecutive batches).
        let alloc_input_buffer = || Vec::<f64>::with_capacity(resampler.input_frames_max() * 10);
        let mut input_buffer = vec![alloc_input_buffer(); 2];
        let mut output_buffer = resampler.output_buffer_allocate(true);

        // Used to fill missing samples and determine batch pts
        let mut send_samples = 0;
        let mut first_batch_pts = None;
        let mut previous_end_pts = None;

        for event in receiver {
            let PipelineEvent::Data(decoded_samples) = event else {
                break
            };
            if decoded_samples.sample_rate != input_sample_rate {
                error!(
                    "Resampler received samples with wrong sample rate. Expected sample rate: {}, received: {}",
                    input_sample_rate,
                    decoded_samples.sample_rate
                );
            }

            append_to_input_buffer(&mut input_buffer, &decoded_samples, &mut previous_end_pts);
            let first_batch_pts = *first_batch_pts.get_or_insert(decoded_samples.start_pts);

            while resampler.input_frames_next() <= input_buffer[0].len() {
                match resampler.process_into_buffer(&input_buffer, &mut output_buffer, None) {
                    Ok((used_input_samples, produced_samples)) => {
                        let samples =
                            Arc::new(read_output_buffer(&output_buffer, produced_samples));
                        let start_pts =
                            batch_pts(first_batch_pts, output_sample_rate, send_samples);
                        let input_samples =
                            InputSamples::new(samples, start_pts, output_sample_rate);

                        drop_input_samples(&mut input_buffer, used_input_samples);
                        send_samples += input_samples.len();
                        if sender.send(PipelineEvent::Data(input_samples)).is_err() {
                            debug!("Failed to send resampled samples.")
                        }
                    }
                    Err(err) => {
                        error!("Resampling error: {}", err)
                    }
                }
            }
        }

        sender.send(PipelineEvent::EOS).unwrap();
    }
}

fn append_to_input_buffer(
    input_buffer: &mut [Vec<f64>],
    decoded_samples: &DecodedSamples,
    previous_end_pts: &mut Option<Duration>,
) {
    const PTS_COMPARE_ERROR_MARGIN: Duration = Duration::from_nanos(100);
    if let Some(end_pts) = previous_end_pts {
        if decoded_samples.start_pts > *end_pts + PTS_COMPARE_ERROR_MARGIN {
            debug!("Filling missing samples in resampler.");
            fill_missing_samples(
                input_buffer,
                decoded_samples.start_pts.saturating_sub(*end_pts),
                decoded_samples.sample_rate,
            );
        }
    }
    for (l, r) in decoded_samples.samples.iter().cloned() {
        input_buffer[0].push(pcm_i16_to_f64(l));
        input_buffer[1].push(pcm_i16_to_f64(r));
    }

    *previous_end_pts = Some(decoded_samples.end_pts());
}

fn fill_missing_samples(
    input_buffer: &mut [Vec<f64>],
    missing_duration: Duration,
    sample_rate: u32,
) {
    let missing_samples_count = (missing_duration.as_secs_f64() * sample_rate as f64) as usize;
    let missing_samples = (0..missing_samples_count).map(|_| 0.0);
    input_buffer[0].extend(missing_samples.clone());
    input_buffer[1].extend(missing_samples);
}

fn read_output_buffer(output_buffer: &[Vec<f64>], output_samples: usize) -> Vec<(i16, i16)> {
    let left_channel_iter = output_buffer[0][0..output_samples].iter().cloned();
    let right_channel_iter = output_buffer[1][0..output_samples].iter().cloned();

    left_channel_iter
        .zip(right_channel_iter)
        .map(|(l, r)| (pcm_f64_to_i16(l), pcm_f64_to_i16(r)))
        .collect()
}

fn batch_pts(first_batch_pts: Duration, sample_rate: u32, send_samples: usize) -> Duration {
    let time_before_batch = Duration::from_secs_f64(send_samples as f64 / sample_rate as f64);
    first_batch_pts + time_before_batch
}

fn drop_input_samples(input_buffer: &mut [Vec<f64>], used_samples: usize) {
    input_buffer[0].drain(0..used_samples);
    input_buffer[1].drain(0..used_samples);
}

fn pcm_i16_to_f64(val: i16) -> f64 {
    val as f64 / i16::MAX as f64
}

fn pcm_f64_to_i16(val: f64) -> i16 {
    let mapped_to_i16_range = val * i16::MAX as f64;
    mapped_to_i16_range
        .min(i16::MAX as f64)
        .max(i16::MIN as f64) as i16
}
