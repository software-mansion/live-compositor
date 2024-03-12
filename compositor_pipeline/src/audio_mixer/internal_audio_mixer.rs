use std::{
    cmp::{max, min},
    collections::{HashMap, VecDeque},
    time::Duration,
};

use compositor_render::{error::UpdateSceneError, InputId, OutputId};
use log::error;

use crate::audio_mixer::types::{InputParams, MixingStrategy};

use super::types::{
    AudioChannels, AudioMixingParams, AudioSamples, InputSamples, InputSamplesSet, OutputSamples,
    OutputSamplesSet,
};

#[derive(Debug)]
struct OutputInfo {
    audio: AudioMixingParams,
    channels: AudioChannels,
}

#[derive(Debug)]
struct InputInfo {
    samples_left: VecDeque<i16>,
    samples_right: VecDeque<i16>,
    popped_samples: u64,
    start_pts: Option<Duration>,
}

impl InputInfo {
    pub fn enqueue(&mut self, input_samples: &InputSamples, output_sample_rate: u32) {
        let start_pts = *self.start_pts.get_or_insert(input_samples.start_pts);
        let expected_samples_before = input_samples
            .start_pts
            .saturating_sub(start_pts)
            .as_secs_f64()
            * output_sample_rate as f64;
        let missing_samples =
            expected_samples_before as u64 - self.popped_samples - self.samples_left.len() as u64;
        if missing_samples > 10 {
            for _ in 0..missing_samples {
                self.samples_left.push_back(0);
                self.samples_right.push_back(0);
            }
        }

        self.samples_left.extend(input_samples.left.iter());
        self.samples_right.extend(input_samples.right.iter());
    }

    pub fn pop(&mut self, samples_count: usize) -> (Vec<i16>, Vec<i16>) {
        let missing_samples = if samples_count > self.samples_left.len() {
            samples_count - self.samples_left.len()
        } else {
            0
        };

        for _ in 0..missing_samples {
            self.samples_left.push_back(0);
            self.samples_right.push_back(0);
        }

        self.popped_samples += samples_count as u64;

        (
            self.samples_left.drain(0..samples_count).collect(),
            self.samples_right.drain(0..samples_count).collect(),
        )
    }
}

#[derive(Debug)]
pub(super) struct InternalAudioMixer {
    inputs: HashMap<InputId, InputInfo>,
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

    pub fn mix_samples(&mut self, samples_set: InputSamplesSet) -> OutputSamplesSet {
        let InputSamplesSet {
            samples,
            start_pts,
            end_pts,
        } = samples_set;

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
                    let output_samples = OutputSamples { samples, start_pts };
                    (output_id.clone(), output_samples)
                })
                .collect(),
        )
    }

    fn pop_samples(&mut self, samples_count: usize) -> HashMap<InputId, (Vec<i16>, Vec<i16>)> {
        self.inputs
            .iter_mut()
            .map(|(input_id, input_info)| (input_id.clone(), input_info.pop(samples_count)))
            .collect()
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

    fn save_samples(&mut self, samples: &HashMap<InputId, Vec<InputSamples>>) {
        for (input_id, input_samples_vec) in samples {
            let Some(input_info) = self.inputs.get_mut(input_id) else {
                error!("Audio mixer received samples for unregistered input");
                return;
            };
            for input_samples in input_samples_vec {
                input_info.enqueue(input_samples, self.output_sample_rate);
            }
        }
    }
}

fn sum_samples<'a, I: Iterator<Item = &'a InputParams>>(
    input_samples: &HashMap<InputId, (Vec<i16>, Vec<i16>)>,
    samples_count: usize,
    inputs: I,
) -> (Vec<i64>, Vec<i64>) {
    let mut summed_samples_left = vec![0i64; samples_count];
    let mut summed_samples_right = vec![0i64; samples_count];

    for input_params in inputs {
        let Some((input_left, input_right)) = input_samples.get(&input_params.input_id) else {
            continue;
        };
        summed_samples_left
            .iter_mut()
            .zip(input_left.iter())
            .for_each(|(sum, s)| *sum += *s as i64);

        summed_samples_right
            .iter_mut()
            .zip(input_right.iter())
            .for_each(|(sum, s)| *sum += *s as i64);
    }

    (summed_samples_left, summed_samples_right)
}

fn mix(
    input_samples: &HashMap<InputId, (Vec<i16>, Vec<i16>)>,
    output_info: &OutputInfo,
    samples_count: usize,
) -> AudioSamples {
    let (summed_left, summed_right) = sum_samples(
        input_samples,
        samples_count,
        output_info.audio.inputs.iter(),
    );

    let (left, right): (Vec<i16>, Vec<i16>) = match output_info.audio.mixing_strategy {
        MixingStrategy::SumClip => {
            let clip = |s: i64| min(max(s, i16::MIN as i64), i16::MAX as i64) as i16;

            let left = summed_left.into_iter().map(clip).collect();
            let right = summed_right.into_iter().map(clip).collect();

            (left, right)
        }
        MixingStrategy::SumScale => {
            let max_abs = |v: &Vec<i64>| v.iter().map(|s| s.abs()).max().unwrap();
            let scaling_factor = |max_abs: i64| {
                if max_abs > i16::MAX as i64 {
                    max_abs as f64 / i16::MAX as f64
                } else {
                    1.0
                }
            };

            let clip = |s: f64| {
                if s >= i16::MAX as f64 {
                    i16::MAX
                } else if s <= i16::MIN as f64 {
                    i16::MIN
                } else {
                    s as i16
                }
            };

            let scale = |v: Vec<i64>| {
                let scaling_factor = scaling_factor(max_abs(&v));
                v.into_iter()
                    .map(|s| s as f64 * scaling_factor)
                    .map(clip)
                    .collect()
            };

            (scale(summed_left), scale(summed_right))
        }
    };

    match output_info.channels {
        AudioChannels::Mono => AudioSamples::Mono(
            left.into_iter()
                .zip(right)
                .map(|(l, r)| ((l as i32 + r as i32) / 2) as i16)
                .collect(),
        ),
        AudioChannels::Stereo => AudioSamples::Stereo(left.into_iter().zip(right).collect()),
    }
}
