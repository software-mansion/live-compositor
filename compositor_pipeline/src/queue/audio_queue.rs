use std::{
    cmp::min,
    collections::{HashMap, VecDeque},
    time::Duration,
};

use compositor_render::{AudioSamples, AudioSamplesBatch, AudioSamplesSet, InputId};

use crate::pipeline::AudioChannels;

use super::QueueError;

pub struct AudioQueue {
    input_queues: HashMap<InputId, InputQueue>,
    timestamp_offsets: HashMap<InputId, Duration>,
}

impl AudioQueue {
    pub fn new() -> Self {
        AudioQueue {
            input_queues: HashMap::new(),
            timestamp_offsets: HashMap::new(),
        }
    }

    pub fn add_input(&mut self, input_id: InputId, channels: AudioChannels, sample_rate: u32) {
        let input_queue = InputQueue::new(channels, sample_rate);
        self.input_queues.insert(input_id, input_queue);
    }

    pub fn remove_input(&mut self, input_id: &InputId) {
        self.input_queues.remove(input_id);
    }

    pub fn enqueue_samples(
        &mut self,
        input_id: InputId,
        samples_batch: AudioSamplesBatch,
    ) -> Result<(), QueueError> {
        let Some(input_queue) = self.input_queues.get_mut(&input_id) else {
            return Err(QueueError::UnknownInputId(input_id));
        };
        let offset = *self
            .timestamp_offsets
            .entry(input_id)
            .or_insert_with(|| samples_batch.pts);

        input_queue.enqueue_samples(samples_batch.samples, samples_batch.pts - offset)?;

        Ok(())
    }

    pub fn pop_samples_set(&mut self, pts: Duration, length: Duration) -> AudioSamplesSet {
        let mut samples = HashMap::new();
        self.input_queues
            .iter_mut()
            .for_each(|(input_id, input_queue)| {
                samples.insert(input_id.clone(), input_queue.pop(length));
            });

        AudioSamplesSet {
            samples,
            pts,
            length,
        }
    }
}

enum SamplesQueue {
    Mono(VecDeque<i16>),
    Stereo(VecDeque<(i16, i16)>),
}

impl SamplesQueue {
    pub fn new(channels: AudioChannels) -> Self {
        match channels {
            AudioChannels::Mono => SamplesQueue::Mono(VecDeque::new()),
            AudioChannels::Stereo => SamplesQueue::Stereo(VecDeque::new()),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            SamplesQueue::Mono(samples) => samples.len(),
            SamplesQueue::Stereo(samples) => samples.len(),
        }
    }
}

struct InputQueue {
    queue: SamplesQueue,
    popped_samples: u64,
    sample_rate: u32,
}

impl InputQueue {
    pub fn new(channels: AudioChannels, sample_rate: u32) -> Self {
        InputQueue {
            queue: SamplesQueue::new(channels),
            popped_samples: 0,
            sample_rate,
        }
    }

    pub fn enqueue_samples(
        &mut self,
        samples: AudioSamples,
        pts: Duration,
    ) -> Result<(), QueueError> {
        let expected_previous_samples = (pts.as_secs_f64() * self.sample_rate as f64) as i64;
        let missing_samples =
            expected_previous_samples - self.popped_samples as i64 - self.queue.len() as i64;

        match &mut self.queue {
            SamplesQueue::Mono(queue) => {
                let AudioSamples::Mono(samples) = samples else {
                    return Err(QueueError::MismatchedSamplesChannels {
                        expected: AudioChannels::Mono,
                        received: samples
                    });
                };
                // To account for calculation precision errors
                if missing_samples > 5 {
                    queue.resize(queue.len() + missing_samples as usize, 0);
                }
                queue.extend(samples.iter());
            }
            SamplesQueue::Stereo(queue) => {
                let AudioSamples::Stereo(samples) = samples else {
                    return Err(QueueError::MismatchedSamplesChannels {
                        expected: AudioChannels::Stereo,
                        received: samples
                    });
                };

                if missing_samples > 5 {
                    queue.resize(queue.len() + missing_samples as usize, (0, 0));
                }
                queue.extend(samples.iter());
            }
        }

        Ok(())
    }

    pub fn pop(&mut self, length: Duration) -> AudioSamples {
        let samples_count = (length.as_secs_f64() * self.sample_rate as f64) as usize;
        let samples_to_take = min(samples_count, self.queue.len());

        match &mut self.queue {
            SamplesQueue::Mono(queue) => {
                let samples = queue.drain(0..samples_to_take).collect::<Vec<i16>>();
                self.popped_samples += samples.len() as u64;
                AudioSamples::Mono(samples)
            }
            SamplesQueue::Stereo(queue) => {
                let samples = queue.drain(0..samples_to_take).collect::<Vec<(i16, i16)>>();
                self.popped_samples += samples.len() as u64;
                AudioSamples::Stereo(samples)
            }
        }
    }
}
