use std::{
    cmp::{max, min},
    collections::{HashMap, VecDeque},
    time::Duration,
};

use compositor_render::{error::UpdateSceneError, InputId, OutputId};
use log::error;

use crate::audio_mixer::{InputParams, MixingStrategy};

use super::types::{
    AudioChannels, AudioMixingParams, AudioSamples, InputSamples, InputSamplesSet, OutputSamples,
    OutputSamplesSet,
};

#[derive(Debug)]
pub(super) struct InternalAudioMixer {
    inputs: HashMap<InputId, InputState>,
    outputs: HashMap<OutputId, OutputInfo>,
    output_sample_rate: u32,
}

impl InternalAudioMixer {
    pub fn new(output_sample_rate: u32) -> Self {
        Self {
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            output_sample_rate,
        }
    }

    pub fn register_input(&mut self, input_id: InputId) {
        self.inputs.insert(
            input_id,
            InputState {
                samples: VecDeque::new(),
                popped_samples: 0,
                start_pts: None,
                last_enqueued_pts: None,
            },
        );
    }

    pub fn unregister_input(&mut self, input_id: &InputId) {
        self.inputs.remove(input_id);
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

    pub fn mix_samples(&mut self, samples_set: InputSamplesSet) -> OutputSamplesSet {
        let InputSamplesSet {
            samples,
            start_pts,
            end_pts,
        } = samples_set;

        // Input samples are saved into input state and popped from it.
        // I tried more functional approach before,
        // that merged batches from inputs in the frame, but floating point
        // arithmetics errors in indexes produced noticeable sound distortions.
        // Moreover, this simplifies logic for filling missing samples.
        self.save_samples(&samples);
        let samples_count = (end_pts.saturating_sub(start_pts).as_secs_f64()
            * self.output_sample_rate as f64)
            .round() as usize;

        let input_samples = self.pop_samples(samples_count);

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

    fn save_samples(&mut self, samples: &HashMap<InputId, Vec<InputSamples>>) {
        for (input_id, input_samples_vec) in samples {
            let Some(input_state) = self.inputs.get_mut(input_id) else {
                error!("Audio mixer received samples for unregistered input");
                return;
            };
            for input_samples in input_samples_vec {
                input_state.enqueue(input_samples, self.output_sample_rate);
            }
        }
    }

    fn pop_samples(&mut self, samples_count: usize) -> HashMap<InputId, Vec<(i16, i16)>> {
        self.inputs
            .iter_mut()
            .map(|(input_id, input_state)| (input_id.clone(), input_state.pop(samples_count)))
            .collect()
    }
}

#[derive(Debug)]
struct OutputInfo {
    audio: AudioMixingParams,
    channels: AudioChannels,
}

#[derive(Debug)]
struct InputState {
    samples: VecDeque<(i16, i16)>,
    popped_samples: u64,
    start_pts: Option<Duration>,
    last_enqueued_pts: Option<Duration>,
}

impl InputState {
    pub fn enqueue(&mut self, input_samples: &InputSamples, output_sample_rate: u32) {
        let start_pts = *self.start_pts.get_or_insert(input_samples.start_pts);
        match self.last_enqueued_pts {
            Some(last_pts) => {
                if last_pts < input_samples.start_pts {
                    self.last_enqueued_pts = Some(input_samples.start_pts);
                } else {
                    return;
                }
            }
            None => self.last_enqueued_pts = Some(input_samples.start_pts),
        }

        let expected_samples_before = input_samples
            .start_pts
            .saturating_sub(start_pts)
            .as_secs_f64()
            * output_sample_rate as f64;
        let missing_samples = (expected_samples_before as u64)
            .saturating_sub(self.popped_samples)
            .saturating_sub(self.samples.len() as u64);
        if missing_samples > 10 {
            for _ in 0..missing_samples {
                self.samples.push_back((0, 0));
            }
        }

        self.samples.extend(input_samples.samples.iter());
    }

    pub fn pop(&mut self, samples_count: usize) -> Vec<(i16, i16)> {
        let missing_samples = if samples_count > self.samples.len() {
            samples_count - self.samples.len()
        } else {
            0
        };

        for _ in 0..missing_samples {
            self.samples.push_back((0, 0));
        }

        self.popped_samples += samples_count as u64;
        self.samples.drain(0..samples_count).collect()
    }
}

/// Mix input samples accordingly to provided specification.
fn mix(
    input_samples: &HashMap<InputId, Vec<(i16, i16)>>,
    output_info: &OutputInfo,
    samples_count: usize,
) -> AudioSamples {
    /// Clips sample to i16 PCM range
    fn clip_to_i16(sample: i64) -> i16 {
        min(max(sample, i16::MIN as i64), i16::MAX as i64) as i16
    }

    let summed_samples = sum_samples(
        input_samples,
        samples_count,
        output_info.audio.inputs.iter(),
    );

    let mixed: Vec<(i16, i16)> = match output_info.audio.mixing_strategy {
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
                    .map(|(l, r)| (l.abs().max(r.abs())))
                    .max()
                    .unwrap();
                if max_abs > i16::MAX as i64 {
                    max_abs as f64 / i16::MAX as f64
                } else {
                    1.0
                }
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

    convert_channels(mixed, output_info.channels)
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
        summed_samples
            .iter_mut()
            .zip(input_samples.iter())
            .for_each(|(sum, s)| {
                sum.0 += (s.0 as f64 * input_params.volume as f64) as i64;
                sum.1 += (s.1 as f64 * input_params.volume as f64) as i64;
            })
    }

    summed_samples
}

fn convert_channels(samples: Vec<(i16, i16)>, channels: AudioChannels) -> AudioSamples {
    match channels {
        AudioChannels::Mono => AudioSamples::Mono(
            samples
                .into_iter()
                // Convert to i32 to avoid additions overflows
                .map(|(l, r)| ((l as i32 + r as i32) / 2) as i16)
                .collect(),
        ),
        AudioChannels::Stereo => AudioSamples::Stereo(samples),
    }
}
