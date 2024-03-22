use std::sync::Arc;
use std::time::Duration;

use log::{debug, error};
use rubato::{FftFixedOut, Resampler as _};

use crate::{audio_mixer::InputSamples, error::DecoderInitError};

use super::DecodedSamples;

const SAMPLE_BATCH_DURATION: Duration = Duration::from_millis(20);

pub(super) enum Resampler {
    Passthrough(PassthroughResampler),
    Fft(Box<FftResampler>),
}

impl Resampler {
    pub fn new(input_sample_rate: u32, output_sample_rate: u32) -> Result<Self, DecoderInitError> {
        if input_sample_rate == output_sample_rate {
            Ok(Self::Passthrough(PassthroughResampler::new(
                input_sample_rate,
                output_sample_rate,
            )))
        } else {
            FftResampler::new(input_sample_rate, output_sample_rate)
                .map(Box::new)
                .map(Self::Fft)
        }
    }

    pub fn resample(&mut self, decoded_samples: DecodedSamples) -> Vec<InputSamples> {
        match self {
            Resampler::Passthrough(resampler) => resampler.resample(decoded_samples),
            Resampler::Fft(resampler) => resampler.resample(decoded_samples),
        }
    }
}

pub(super) struct PassthroughResampler {
    input_sample_rate: u32,
    output_sample_rate: u32,
}

impl PassthroughResampler {
    fn new(input_sample_rate: u32, output_sample_rate: u32) -> Self {
        if input_sample_rate != output_sample_rate {
            error!("Passthrough resampler was used for resampling to different sample rate.")
        }
        Self {
            input_sample_rate,
            output_sample_rate,
        }
    }

    fn resample(&mut self, decoded_samples: DecodedSamples) -> Vec<InputSamples> {
        if decoded_samples.sample_rate != self.input_sample_rate {
            error!("Passthrough resampler received decoded samples in wrong sample rate. Expected {}, actual: {}", self.input_sample_rate, decoded_samples.sample_rate);
            return Vec::new();
        }
        Vec::from([InputSamples::new(
            decoded_samples.samples,
            decoded_samples.start_pts,
            self.output_sample_rate,
        )])
    }
}

pub(super) struct FftResampler {
    input_sample_rate: u32,
    output_sample_rate: u32,
    input_buffer: [Vec<f64>; 2],
    output_buffer: [Vec<f64>; 2],
    resampler: FftFixedOut<f64>,
    send_samples: u64,
    first_batch_pts: Option<Duration>,
    previous_end_pts: Option<Duration>,
}

impl FftResampler {
    fn new(
        input_sample_rate: u32,
        output_sample_rate: u32,
    ) -> Result<FftResampler, DecoderInitError> {
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

        // Input buffer is preallocated, to push input samples and fill missing samples between them.
        // Reallocation happens per every output batch, due to drain from the begging,
        // but this shouldn't have a noticeable performance impact and reduce code complexity.
        // This could be done without allocations, but it would complicate this code substantially.
        let input_buffer = [Vec::new(), Vec::new()];

        // Output buffer is preallocated to avoid allocating it on every output batch.
        let output_buffer = [vec![0.0; output_batch_size], vec![0.0; output_batch_size]];

        // Used to fill missing samples and determine batch pts
        let send_samples = 0;
        let first_batch_pts = None;
        let previous_end_pts = None;

        Ok(Self {
            input_sample_rate,
            output_sample_rate,
            input_buffer,
            output_buffer,
            resampler,
            send_samples,
            first_batch_pts,
            previous_end_pts,
        })
    }

    fn resample(&mut self, decoded_samples: DecodedSamples) -> Vec<InputSamples> {
        if decoded_samples.sample_rate != self.input_sample_rate {
            error!(
                "Resampler received samples with wrong sample rate. Expected sample rate: {}, received: {}",
                self.input_sample_rate,
                decoded_samples.sample_rate
            );
        }

        append_to_input_buffer(
            &mut self.input_buffer,
            &decoded_samples,
            &mut self.previous_end_pts,
        );
        let first_batch_pts = *self
            .first_batch_pts
            .get_or_insert(decoded_samples.start_pts);

        let mut resampled = Vec::new();
        while self.resampler.input_frames_next() <= self.input_buffer[0].len() {
            match self.resampler.process_into_buffer(
                &self.input_buffer,
                &mut self.output_buffer,
                None,
            ) {
                Ok((used_input_samples, produced_samples)) => {
                    let samples =
                        Arc::new(read_output_buffer(&self.output_buffer, produced_samples));
                    let start_pts =
                        batch_pts(first_batch_pts, self.output_sample_rate, self.send_samples);
                    let input_samples =
                        InputSamples::new(samples, start_pts, self.output_sample_rate);

                    drop_input_samples(&mut self.input_buffer, used_input_samples);
                    self.send_samples += input_samples.len() as u64;
                    resampled.push(input_samples);
                }
                Err(err) => {
                    error!("Resampling error: {}", err)
                }
            }
        }

        resampled
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

fn batch_pts(first_batch_pts: Duration, sample_rate: u32, send_samples: u64) -> Duration {
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
