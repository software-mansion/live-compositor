use std::sync::Arc;
use std::time::Duration;

use log::{debug, error};
use rubato::{FftFixedOut, Resampler as _};
use tracing::trace;

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
    first_batch_pts: Option<Duration>,
    resampler_input_samples: u64,
    resampler_output_samples: u64,
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

        Ok(Self {
            input_sample_rate,
            output_sample_rate,
            input_buffer,
            output_buffer,
            resampler,
            first_batch_pts: None,
            resampler_input_samples: 0,
            resampler_output_samples: 0,
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
        self.append_to_input_buffer(decoded_samples);

        let mut resampled = Vec::new();
        while self.resampler.input_frames_next() <= self.input_buffer[0].len() {
            let start_pts = self.output_batch_pts();

            match self.resampler.process_into_buffer(
                &self.input_buffer,
                &mut self.output_buffer,
                None,
            ) {
                Ok((used_input_samples, produced_samples)) => {
                    let samples = Arc::new(self.read_output_buffer(produced_samples));
                    let input_samples =
                        InputSamples::new(samples, start_pts, self.output_sample_rate);

                    self.drop_input_samples(used_input_samples);
                    self.resampler_input_samples += used_input_samples as u64;
                    self.resampler_output_samples += produced_samples as u64;
                    resampled.push(input_samples);
                }
                Err(err) => {
                    error!("Resampling error: {}", err)
                }
            }
        }

        trace!(?resampled, "FFT resampler produced samples.");
        resampled
    }

    fn append_to_input_buffer(&mut self, decoded_samples: DecodedSamples) {
        let first_batch_pts = *self
            .first_batch_pts
            .get_or_insert(decoded_samples.start_pts);

        let input_duration = decoded_samples.start_pts.saturating_sub(first_batch_pts);
        let expected_samples =
            (input_duration.as_secs_f64() * self.input_sample_rate as f64) as u64;
        let actual_samples = self.resampler_input_samples + self.input_buffer[0].len() as u64;

        const SAMPLES_COMPARE_ERROR_MARGIN: u64 = 1;
        if expected_samples > actual_samples + SAMPLES_COMPARE_ERROR_MARGIN {
            let filling_samples = expected_samples - actual_samples;
            debug!("Filling {} missing samples in resampler", filling_samples);
            for _ in 0..filling_samples {
                self.input_buffer[0].push(0.0);
                self.input_buffer[1].push(0.0);
            }
        }

        for (l, r) in decoded_samples.samples.iter().cloned() {
            self.input_buffer[0].push(pcm_i16_to_f64(l));
            self.input_buffer[1].push(pcm_i16_to_f64(r));
        }
    }

    fn read_output_buffer(&mut self, output_samples: usize) -> Vec<(i16, i16)> {
        let left_channel_iter = self.output_buffer[0][0..output_samples].iter().cloned();
        let right_channel_iter = self.output_buffer[1][0..output_samples].iter().cloned();

        left_channel_iter
            .zip(right_channel_iter)
            .map(|(l, r)| (pcm_f64_to_i16(l), pcm_f64_to_i16(r)))
            .collect()
    }

    fn drop_input_samples(&mut self, used_samples: usize) {
        self.input_buffer[0].drain(0..used_samples);
        self.input_buffer[1].drain(0..used_samples);
    }

    fn output_batch_pts(&mut self) -> Duration {
        let send_audio_duration = Duration::from_secs_f64(
            self.resampler_output_samples as f64 / self.output_sample_rate as f64,
        );
        self.first_batch_pts.unwrap() + send_audio_duration
    }
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
