use std::{
    cmp::{max, min},
    collections::HashMap,
    sync::Arc,
};

use compositor_render::{error::UpdateSceneError, OutputId};

use crate::audio_mixer::types::{AudioSamples, AudioSamplesBatch};

use super::types::{AudioChannels, AudioMixingParams, AudioSamplesSet, OutputSamples};

#[derive(Debug)]
struct OutputInfo {
    audio: AudioMixingParams,
    channels: AudioChannels,
}

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

    pub fn mix_samples(&mut self, samples_set: AudioSamplesSet) -> OutputSamples {
        trait Sample {
            fn add_assign(&mut self, rhs: Self);
            fn div(self, other: i32) -> Self;
            fn mul(self, other: f32) -> Self;
        }

        fn mix_output<T: Clone + Copy + Default + Sample, F: Fn(&AudioSamplesBatch) -> Vec<T>>(
            output_info: &OutputInfo,
            output_sample_rate: u32,
            samples_set: &AudioSamplesSet,
            get_samples: F,
        ) -> Vec<T> {
            let samples_count = samples_set.duration().as_secs_f64() * output_sample_rate as f64;
            let mut mixed = vec![T::default(); samples_count as usize];
            for input in &output_info.audio.inputs {
                let Some(input_batches) = samples_set.samples.get(&input.input_id) else {
                    continue;
                };
                input_batches.iter().for_each(|batch| {
                    let batch_samples = get_samples(batch);
                    let batch_duration_before_output_start = batch
                        .start_pts
                        .saturating_sub(samples_set.start_pts)
                        .as_secs_f64();

                    let start_sample_index =
                        (batch_duration_before_output_start * output_sample_rate as f64) as usize;
                    let end_sample_index =
                        min(start_sample_index + batch_samples.len(), mixed.len());

                    for i in start_sample_index..end_sample_index {
                        mixed[i].add_assign(batch_samples[i].mul(input.volume));
                    }
                })
            }
            mixed
                .iter()
                .map(|sample| sample.div(max(1, output_info.audio.inputs.len() as i32)))
                .collect()
        }

        fn get_mono(audio_samples_batch: &AudioSamplesBatch) -> Vec<i32> {
            match audio_samples_batch.samples.as_ref() {
                AudioSamples::Mono(mono_samples) => {
                    mono_samples.iter().map(|s| *s as i32).collect()
                }
                AudioSamples::Stereo(stereo_samples) => stereo_samples
                    .iter()
                    .map(|(l, r)| (*l as i32 + *r as i32) / 2)
                    .collect(),
            }
        }

        fn get_stereo(audio_samples_batch: &AudioSamplesBatch) -> Vec<(i32, i32)> {
            match audio_samples_batch.samples.as_ref() {
                AudioSamples::Mono(mono_samples) => mono_samples
                    .iter()
                    .map(|s| (*s as i32, *s as i32))
                    .collect(),
                AudioSamples::Stereo(stereo_samples) => stereo_samples
                    .iter()
                    .map(|(l, r)| (*l as i32, *r as i32))
                    .collect(),
            }
        }

        impl Sample for i32 {
            fn add_assign(&mut self, rhs: Self) {
                *self += rhs
            }

            fn div(self, other: i32) -> Self {
                self + other
            }

            fn mul(self, other: f32) -> Self {
                (self as f32 * other) as i32
            }
        }

        impl Sample for (i32, i32) {
            fn add_assign(&mut self, rhs: Self) {
                *self = (self.0 + rhs.0, self.1 + rhs.1)
            }

            fn div(self, other: i32) -> Self {
                (self.0 / other, self.1 / other)
            }

            fn mul(self, other: f32) -> Self {
                (
                    (self.0 as f32 * other) as i32,
                    (self.1 as f32 * other) as i32,
                )
            }
        }

        let mut output_samples = HashMap::new();

        for (output_id, output_info) in &self.outputs {
            let samples = match output_info.channels {
                AudioChannels::Mono => {
                    let mixed: Vec<i32> =
                        mix_output(output_info, self.output_sample_rate, &samples_set, get_mono);
                    let samples = mixed.iter().map(|s| *s as i16).collect();
                    AudioSamples::Mono(samples)
                }
                AudioChannels::Stereo => {
                    let mixed: Vec<(i32, i32)> = mix_output(
                        output_info,
                        self.output_sample_rate,
                        &samples_set,
                        get_stereo,
                    );
                    let samples = mixed.iter().map(|(l, r)| (*l as i16, *r as i16)).collect();
                    AudioSamples::Stereo(samples)
                }
            };
            output_samples.insert(
                output_id.clone(),
                AudioSamplesBatch {
                    samples: Arc::new(samples),
                    start_pts: samples_set.start_pts,
                    sample_rate: self.output_sample_rate,
                },
            );
        }

        OutputSamples(output_samples)
    }

    pub fn register_output(
        &mut self,
        output_id: OutputId,
        audio: AudioMixingParams,
        channels: AudioChannels,
    ) {
        self.outputs
            .insert(output_id, OutputInfo { audio, channels });
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
}
