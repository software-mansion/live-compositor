use std::{collections::HashMap, time::Duration};

use compositor_render::{error::UpdateSceneError, InputId, OutputId};

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
        let samples_count = Self::samples_in_frame(
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

    fn frame_input_samples(
        start_pts: Duration,
        end_pts: Duration,
        samples: Vec<InputSamples>,
        sample_rate: u32,
    ) -> Vec<(i16, i16)> {
        let mut samples_in_frame = Vec::new();

        samples
            .into_iter()
            .fold(start_pts, |last_end_pts, input_samples| {
                // Filling missing samples before this batch
                let time_since_last_end = input_samples.start_pts.saturating_sub(last_end_pts);
                let missing_samples =
                    (time_since_last_end.as_secs_f64() * sample_rate as f64).floor() as usize;
                if missing_samples > 1 {
                    Self::push_missing_samples(&mut samples_in_frame, missing_samples);
                };
                // The amount of time that should be removed from the beginning of the batch
                let time_to_remove_from_start = start_pts.saturating_sub(input_samples.start_pts);
                let start_index =
                    (time_to_remove_from_start.as_secs_f64() * sample_rate as f64).floor() as usize;

                // Appending batch samples in frame
                let time_to_remove_from_end = input_samples.end_pts.saturating_sub(end_pts);
                let samples_after_frame =
                    (time_to_remove_from_end.as_secs_f64() * sample_rate as f64).ceil() as usize;
                let end_index = input_samples.len() - samples_after_frame;

                samples_in_frame.extend(input_samples.samples[start_index..end_index].iter());

                input_samples.end_pts
            });
        // Appending samples missing in [last_end_pts, ]
        let missing_samples = Self::samples_in_frame(start_pts, end_pts, sample_rate);
        Self::push_missing_samples(&mut samples_in_frame, missing_samples);

        samples_in_frame
    }

    fn samples_in_frame(start: Duration, end: Duration, sample_rate: u32) -> usize {
        (end.saturating_sub(start).as_nanos() * sample_rate as u128 / 1_000_000_000) as usize
    }

    fn push_missing_samples(samples_buffer: &mut Vec<(i16, i16)>, samples_count: usize) {
        let filling_samples = (0..samples_count).map(|_| (0i16, 0i16));
        samples_buffer.extend(filling_samples);
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
