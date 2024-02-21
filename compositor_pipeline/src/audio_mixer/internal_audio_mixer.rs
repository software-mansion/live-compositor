use std::{cmp::max, collections::HashMap, sync::Arc, time::Duration};

use compositor_render::{
    error::UpdateSceneError, scene::AudioComposition, AudioChannels, AudioSamples,
    AudioSamplesBatch, AudioSamplesSet, InputId, OutputId, OutputSamples,
};

#[derive(Debug)]
struct OutputInfo {
    composition: AudioComposition,
    sample_rate: u32,
    channels: AudioChannels,
    // To avoid aggregating rounding error while calculating next lengths of mixed batches
    first_pts: Option<Duration>,
    mixed_samples: usize,
}

#[derive(Debug)]
pub(super) struct InternalAudioMixer {
    outputs: HashMap<OutputId, OutputInfo>,
}

impl InternalAudioMixer {
    pub fn new() -> Self {
        InternalAudioMixer {
            outputs: HashMap::new(),
        }
    }

    pub fn register_output(
        &mut self,
        output_id: OutputId,
        sample_rate: u32,
        channels: AudioChannels,
        initial_composition: AudioComposition,
    ) {
        self.outputs.insert(
            output_id,
            OutputInfo {
                composition: initial_composition,
                sample_rate,
                channels,
                first_pts: None,
                mixed_samples: 0,
            },
        );
    }

    pub fn unregister_output(&mut self, output_id: &OutputId) {
        self.outputs.remove(output_id);
    }

    pub fn update_output(
        &mut self,
        output_id: OutputId,
        composition: AudioComposition,
    ) -> Result<(), UpdateSceneError> {
        let Some(output_info) = self.outputs.get_mut(&output_id) else {
            return Err(UpdateSceneError::OutputNotRegistered(output_id));
        };

        output_info.composition = composition;
        Ok(())
    }

    pub fn mix_samples(&mut self, samples_set: AudioSamplesSet) -> OutputSamples {
        let input_samples = samples_set
            .samples
            .iter()
            .map(|(input_id, batches)| {
                (
                    input_id.clone(),
                    Self::join_consecutive_batches_for_one_input(batches),
                )
            })
            .collect::<HashMap<InputId, Option<AudioSamplesBatch>>>();

        OutputSamples(
            self.outputs
                .iter_mut()
                .map(|(output_id, output_info)| {
                    let samples = Self::mix_output_samples(
                        output_info,
                        &input_samples,
                        samples_set.start_pts,
                        samples_set.end_pts(),
                    );
                    (
                        output_id.clone(),
                        AudioSamplesBatch {
                            samples: Arc::new(samples),
                            start_pts: samples_set.start_pts,
                            sample_rate: output_info.sample_rate,
                        },
                    )
                })
                .collect(),
        )
    }

    fn mix_output_samples(
        output_info: &mut OutputInfo,
        input_samples: &HashMap<InputId, Option<AudioSamplesBatch>>,
        start_pts: Duration,
        end_pts: Duration,
    ) -> AudioSamples {
        // i32 is used for mixing, to avoid int overflows
        // It's easier to just collect it in Vec then use Box<dyn Iterator> and deal with lifetimes
        fn mono_samples(batch: &AudioSamplesBatch) -> Vec<i32> {
            match batch.samples.as_ref() {
                AudioSamples::Mono(samples) => samples.iter().map(|s| *s as i32).collect(),
                AudioSamples::Stereo(samples) => samples
                    .iter()
                    .map(|(l, r)| (*l as i32 + *r as i32) / 2)
                    .collect(),
            }
        }

        fn stereo_samples(batch: &AudioSamplesBatch) -> Vec<(i32, i32)> {
            match batch.samples.as_ref() {
                AudioSamples::Mono(samples) => {
                    samples.iter().map(|s| (*s as i32, *s as i32)).collect()
                }
                AudioSamples::Stereo(samples) => samples
                    .iter()
                    .map(|(l, r)| (*l as i32, *r as i32))
                    .collect(),
            }
        }

        trait ExtSampleOps {
            fn add(&self, other: Self) -> Self;
            fn div(&self, counter: i32) -> Self;
        }

        impl ExtSampleOps for i32 {
            fn add(&self, other: Self) -> Self {
                self + other
            }

            fn div(&self, counter: i32) -> Self {
                self / max(counter, 1)
            }
        }

        impl ExtSampleOps for (i32, i32) {
            fn add(&self, other: Self) -> Self {
                (self.0 + other.0, self.1 + other.1)
            }

            fn div(&self, counter: i32) -> Self {
                (self.0 / max(counter, 1), self.1 / max(counter, 1))
            }
        }

        /// Mix samples from inputs in mixing buffer
        /// This function avoids code duplication for different sample types
        fn mix<SumSample: Sized + Default + Clone + Copy + ExtSampleOps, F>(
            mixing_buffer: &mut [SumSample],
            // Counts hwo many samples have been summed in mixing_buffer[i]
            counter: &mut [i32],
            output_info: &OutputInfo,
            get_samples: F,
            start_pts: Duration,
            input_samples: &HashMap<InputId, Option<AudioSamplesBatch>>,
        ) where
            F: Fn(&AudioSamplesBatch) -> Vec<SumSample>,
        {
            // Sums inputs in mixing buffer
            output_info
                .composition
                .mixed_inputs
                .iter()
                .filter_map(|input_id| match input_samples.get(input_id) {
                    Some(Some(input_batch)) => Some(input_batch),
                    _ => None,
                })
                .for_each(|input_batch| {
                    get_samples(input_batch)
                        .iter()
                        .enumerate()
                        .for_each(|(index, sample)| {
                            let sample_pts = input_batch.start_pts
                                + Duration::from_secs_f64(
                                    index as f64 / input_batch.sample_rate as f64,
                                );
                            let sample_index = (sample_pts.saturating_sub(start_pts).as_secs_f64()
                                * output_info.sample_rate as f64)
                                as usize;
                            if sample_index < mixing_buffer.len() && sample_pts > start_pts {
                                mixing_buffer[sample_index] =
                                    mixing_buffer[sample_index].add(*sample);
                                counter[sample_index] += 1;
                            }
                        })
                });

            // Takes average of inputs
            counter
                .iter()
                .enumerate()
                .for_each(|(index, count)| mixing_buffer[index] = mixing_buffer[index].div(*count));
        }

        let first_pts = match output_info.first_pts {
            Some(first_pts) => first_pts,
            None => {
                output_info.first_pts = Some(start_pts);
                start_pts
            }
        };

        // Calculated in each mixing to avoid aggregating rounding errors
        let samples_count = (end_pts.saturating_sub(first_pts).as_secs_f64()
            * output_info.sample_rate as f64) as usize
            - output_info.mixed_samples;

        output_info.mixed_samples += samples_count;
        let mut counter = vec![0; samples_count];
        match output_info.channels {
            AudioChannels::Mono => {
                let mut mixing_buffer = vec![0i32; samples_count];
                let get_samples = |batch: &AudioSamplesBatch| mono_samples(batch);
                mix(
                    &mut mixing_buffer,
                    &mut counter,
                    output_info,
                    get_samples,
                    start_pts,
                    input_samples,
                );
                let mixed_samples = mixing_buffer.iter().map(|s| *s as i16).collect();

                AudioSamples::Mono(mixed_samples)
            }
            AudioChannels::Stereo => {
                let mut mixing_buffer = vec![(0i32, 0i32); samples_count];
                let get_samples = |batch: &AudioSamplesBatch| stereo_samples(batch);
                mix(
                    &mut mixing_buffer,
                    &mut counter,
                    output_info,
                    get_samples,
                    start_pts,
                    input_samples,
                );
                let mixed_samples = mixing_buffer
                    .iter()
                    .map(|(l, r)| (*l as i16, *r as i16))
                    .collect();

                AudioSamples::Stereo(mixed_samples)
            }
        }
    }

    /// Joins consecutive batches for a single input, and fills empty spaces with zeroes, if there are any
    fn join_consecutive_batches_for_one_input(
        batches: &[AudioSamplesBatch],
    ) -> Option<AudioSamplesBatch> {
        /// batches shouldn't be empty
        fn join_batches<T: Copy + Default>(
            batches: &[AudioSamplesBatch],
            get_samples: impl Fn(&AudioSamplesBatch) -> Option<&[T]>,
        ) -> Vec<T> {
            let first_batch = batches.first().unwrap();
            batches.iter().fold(Vec::new(), |mut samples, batch| {
                let missing_samples = (batch
                    .start_pts
                    .saturating_sub(first_batch.start_pts)
                    .as_secs_f64()
                    * first_batch.sample_rate as f64)
                    .round() as usize
                    - samples.len();
                // To account for numerical errors
                if missing_samples > 3 {
                    samples.extend(std::iter::repeat(T::default()).take(missing_samples));
                }

                if let Some(batch_samples) = get_samples(batch) {
                    samples.extend(batch_samples.iter().cloned());
                }

                samples
            })
        }

        let first_batch = batches.first()?;
        let samples =
            match first_batch.samples.as_ref() {
                AudioSamples::Mono(_) => AudioSamples::Mono(join_batches(batches, |batch| {
                    match &batch.samples.as_ref() {
                        AudioSamples::Mono(samples) => Some(samples),
                        AudioSamples::Stereo(_) => None,
                    }
                })),
                AudioSamples::Stereo(_) => AudioSamples::Stereo(join_batches(batches, |batch| {
                    match &batch.samples.as_ref() {
                        AudioSamples::Mono(_) => None,
                        AudioSamples::Stereo(samples) => Some(samples),
                    }
                })),
            };

        Some(AudioSamplesBatch {
            samples: Arc::new(samples),
            start_pts: first_batch.start_pts,
            sample_rate: first_batch.sample_rate,
        })
    }
}
