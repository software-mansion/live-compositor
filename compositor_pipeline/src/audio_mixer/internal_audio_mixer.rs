use std::{collections::HashMap, time::Duration};

use compositor_render::{error::UpdateSceneError, InputId, OutputId};
use tracing::warn;

use crate::audio_mixer::{InputParams, MixingStrategy};

use super::{
    types::{
        AudioChannels, AudioMixingParams, AudioSamples, InputSamplesSet, OutputSamples,
        OutputSamplesSet,
    },
    InputSamples,
};

#[derive(Debug)]
pub(super) struct InternalAudioMixer {
    outputs: HashMap<OutputId, OutputInfo>,
    output_sample_rate: u32,
}

impl InternalAudioMixer {
    pub fn new(output_sample_rate: u32) -> Self {
        Self {
            outputs: HashMap::new(),
            output_sample_rate,
        }
    }

    pub fn register_output(
        &mut self,
        output_id: OutputId,
        audio: AudioMixingParams,
        mixing_strategy: MixingStrategy,
        channels: AudioChannels,
    ) {
        self.outputs.insert(
            output_id,
            OutputInfo {
                audio,
                channels,
                mixing_strategy,
            },
        );
    }

    pub fn unregister_output(&mut self, output_id: &OutputId) {
        self.outputs.remove(output_id);
    }

    pub fn update_output(
        &mut self,
        output_id: &OutputId,
        audio: AudioMixingParams,
    ) -> Result<(), UpdateSceneError> {
        match self.outputs.get_mut(output_id) {
            Some(output_info) => {
                output_info.audio = audio;
                Ok(())
            }
            None => Err(UpdateSceneError::OutputNotRegistered(output_id.clone())),
        }
    }

    pub fn mix_samples(&mut self, samples_set: InputSamplesSet) -> OutputSamplesSet {
        let start_pts = samples_set.start_pts;
        let samples_count = Self::expected_samples_count(
            samples_set.start_pts,
            samples_set.end_pts,
            self.output_sample_rate,
        );
        let input_samples = self.merge_fill_input_samples(samples_set);

        OutputSamplesSet(
            self.outputs
                .iter()
                .map(|(output_id, output_info)| {
                    let samples = mix(&input_samples, output_info, samples_count);
                    (output_id.clone(), OutputSamples { samples, start_pts })
                })
                .collect(),
        )
    }

    fn merge_fill_input_samples(
        &self,
        input_samples_set: InputSamplesSet,
    ) -> HashMap<InputId, Vec<(i16, i16)>> {
        input_samples_set
            .samples
            .into_iter()
            .map(|(input_id, input_batch)| {
                let samples = Self::frame_input_samples(
                    input_samples_set.start_pts,
                    input_samples_set.end_pts,
                    input_batch,
                    self.output_sample_rate,
                );

                (input_id, samples)
            })
            .collect()
    }

    // Produce continuous batch of samples for range (start_pts, end_pts).
    //
    // This code assumes that start_pts and end_pts are always numerically correct. Code that
    // generates those timestamps needs to ensure that.
    //
    // To calculate pts of a single sample:
    // - First item in a sample batch starts at the same time as batch PTS.
    // - First item in a sample batch ends `1/sample_rate` seconds latter.
    // - We assume that samples pts values might not be numerically precise
    // - Input and output samples are out of sync, so input pts need to be
    // shifted by an offset.
    //
    // For sample to be included in the output range:
    // - start_pts of a sample >= start_pts of an output batch
    // - end_pts of a sample <= end_pts of an output batch
    // - `=` in above cases means close enough to be a precision related error.
    fn frame_input_samples(
        start_pts: Duration,
        end_pts: Duration,
        samples: Vec<InputSamples>,
        sample_rate: u32,
    ) -> Vec<(i16, i16)> {
        let mut samples_in_frame = Vec::new();

        // Real numerical errors are a lot smaller, but taking max error as 1% of a sample duration
        // seems to be safe enough.
        let max_error = Duration::from_secs_f64(0.01 / sample_rate as f64);

        // Output and input samples have the same sample rate, but they are not synced with each
        // other. We need to calculate an offset between input and output samples. This value
        // should be constant for a specific input, but there is no harm with calculating it every
        // time.
        let sample_offset = samples
            .first()
            .map(|batch| {
                let duration_secs = start_pts.as_secs_f64() - batch.start_pts.as_secs_f64();
                let sample_duration_secs = 1.0 / sample_rate as f64;
                Duration::from_secs_f64(duration_secs.rem_euclid(sample_duration_secs))
            })
            .unwrap_or(Duration::ZERO);

        let time_to_sample_count = |duration: Duration| {
            let sample_count = duration.as_secs_f64() * sample_rate as f64;
            if (sample_count - sample_count.round()).abs() < max_error.as_secs_f64() {
                sample_count.round() as usize
            } else {
                sample_count.floor() as usize
            }
        };

        let last_batch_end_pts = samples.last().map(|sample| sample.end_pts + sample_offset);
        let samples_iter = samples.into_iter().map(|mut sample| {
            sample.start_pts += sample_offset;
            sample.end_pts += sample_offset;
            sample
        });

        for (batch_index, input_samples) in samples_iter.enumerate() {
            let sample_count = samples_in_frame.len();
            let expected_next_sample_start_pts =
                start_pts + Duration::from_secs_f64(sample_count as f64 / sample_rate as f64);

            // potentially fill missing spots
            if expected_next_sample_start_pts + max_error < input_samples.start_pts {
                let missing_time = input_samples
                    .start_pts
                    .saturating_sub(expected_next_sample_start_pts);
                let missing_samples_count = time_to_sample_count(missing_time);
                if missing_samples_count < 1 {
                    warn!(
                        ?missing_time,
                        "Distance between samples is higher than expected."
                    )
                }
                samples_in_frame.extend((0..missing_samples_count).map(|_| (0i16, 0i16)))
            }

            let sample_count = samples_in_frame.len();
            let expected_next_sample_start_pts =
                start_pts + Duration::from_secs_f64(sample_count as f64 / sample_rate as f64);

            // check if we need to drop samples at the beginning
            let mut start_range = 0;
            if expected_next_sample_start_pts > input_samples.start_pts + max_error {
                let time_to_remove_from_start =
                    expected_next_sample_start_pts.saturating_sub(input_samples.start_pts);
                let samples_to_remove_from_start = time_to_sample_count(time_to_remove_from_start);
                if batch_index != 0 {
                    // We should only drop samples in the first batch.
                    warn!(
                        "Decoder produced overlapping samples. Dropping {} samples.",
                        samples_to_remove_from_start
                    );
                }
                start_range = samples_to_remove_from_start;
            };

            // check if we need to drop samples at the end
            let mut end_range = input_samples.len();
            if input_samples.end_pts > end_pts + max_error {
                let desired_duration = end_pts.saturating_sub(expected_next_sample_start_pts);
                let desired_sample_count = time_to_sample_count(desired_duration);
                end_range = start_range + desired_sample_count;
            }

            samples_in_frame.extend(input_samples.samples[start_range..end_range].iter());
        }

        // Fill at the end only if last batch is ending to quickly
        if let Some(pts) = last_batch_end_pts {
            if pts < end_pts + max_error {
                Self::ensure_correct_amount_of_samples(
                    start_pts,
                    end_pts,
                    sample_rate,
                    &mut samples_in_frame,
                );
            }
        }

        Self::check_input_batch(start_pts, end_pts, sample_rate, &samples_in_frame);

        // This call ensures that input buffer has correct amount of samples,
        // but if it needs to do anything it is considered a bug.
        Self::ensure_correct_amount_of_samples(
            start_pts,
            end_pts,
            sample_rate,
            &mut samples_in_frame,
        );

        samples_in_frame
    }

    fn check_input_batch(
        start_pts: Duration,
        end_pts: Duration,
        sample_rate: u32,
        samples: &[(i16, i16)],
    ) {
        let samples_count_times_1e9 =
            end_pts.saturating_sub(start_pts).as_nanos() * sample_rate as u128;
        if samples_count_times_1e9 % 1_000_000_000 != 0 {
            warn!(
                "Duration {:?} is not divisible by sample duration (sample rate: {}).",
                end_pts.saturating_sub(start_pts),
                sample_rate,
            )
        }
        if samples.len() as u128 != samples_count_times_1e9 / 1_000_000_000 {
            warn!(
                "Wrong amount of samples generated. Expected: {}, Actual: {}.",
                samples_count_times_1e9 / 1_000_000_000,
                samples.len()
            );
        }
    }

    fn ensure_correct_amount_of_samples(
        start: Duration,
        end: Duration,
        sample_rate: u32,
        samples_buffer: &mut Vec<(i16, i16)>,
    ) {
        // This is precise as long as (end - start) is divisible by `1/sample_rate`
        let expected_samples_count = Self::expected_samples_count(start, end, sample_rate);
        if expected_samples_count > samples_buffer.len() {
            let missing_samples_count = expected_samples_count - samples_buffer.len();
            let missing_samples = (0..missing_samples_count).map(|_| (0i16, 0i16));
            samples_buffer.extend(missing_samples);
        } else {
            samples_buffer.drain(expected_samples_count..samples_buffer.len());
        }
    }

    fn expected_samples_count(start: Duration, end: Duration, sample_rate: u32) -> usize {
        (end.saturating_sub(start).as_nanos() * sample_rate as u128 / 1_000_000_000) as usize
    }
}

#[derive(Debug)]
struct OutputInfo {
    audio: AudioMixingParams,
    mixing_strategy: MixingStrategy,
    channels: AudioChannels,
}

/// Mix input samples accordingly to provided specification.
fn mix(
    input_samples: &HashMap<InputId, Vec<(i16, i16)>>,
    output_info: &OutputInfo,
    samples_count: usize,
) -> AudioSamples {
    /// Clips sample to i16 PCM range
    fn clip_to_i16(sample: i64) -> i16 {
        sample.min(i16::MAX as i64).max(i16::MIN as i64) as i16
    }

    let summed_samples = sum_samples(
        input_samples,
        samples_count,
        output_info.audio.inputs.iter(),
    );

    let mixed: Vec<(i16, i16)> = match output_info.mixing_strategy {
        MixingStrategy::SumClip => summed_samples
            .into_iter()
            .map(|(l, r)| (clip_to_i16(l), clip_to_i16(r)))
            .collect(),
        MixingStrategy::SumScale => {
            let scaling_factor = {
                // abs panics in debug if val = i64::MIN, but it would require summing so many i16 samples, that it'll never happen.
                // Assumes that summed samples is not empty (therefore unwrap is safe)
                let max_abs = summed_samples
                    .iter()
                    .map(|(l, r)| i64::max(l.abs(), r.abs()))
                    .max()
                    .unwrap();
                f64::max(max_abs as f64 / i16::MAX as f64, 1.0)
            };

            summed_samples
                .into_iter()
                .map(|(l, r)| {
                    (
                        clip_to_i16((l as f64 * scaling_factor) as i64),
                        clip_to_i16((r as f64 * scaling_factor) as i64),
                    )
                })
                .collect()
        }
    };

    match output_info.channels {
        AudioChannels::Mono => AudioSamples::Mono(
            mixed
                .into_iter()
                // Convert to i32 to avoid additions overflows
                .map(|(l, r)| ((l as i32 + r as i32) / 2) as i16)
                .collect(),
        ),
        AudioChannels::Stereo => AudioSamples::Stereo(mixed),
    }
}

/// Sums samples from inputs
fn sum_samples<'a, I: Iterator<Item = &'a InputParams>>(
    input_samples: &HashMap<InputId, Vec<(i16, i16)>>,
    samples_count: usize,
    inputs: I,
) -> Vec<(i64, i64)> {
    let mut summed_samples = vec![(0i64, 0i64); samples_count];

    for input_params in inputs {
        let Some(input_samples) = input_samples.get(&input_params.input_id) else {
            continue;
        };
        for (sum, sample) in summed_samples.iter_mut().zip(input_samples.iter()) {
            sum.0 += (sample.0 as f64 * input_params.volume as f64) as i64;
            sum.1 += (sample.1 as f64 * input_params.volume as f64) as i64;
        }
    }

    summed_samples
}
