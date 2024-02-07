use std::{
    collections::{HashMap, VecDeque},
    time::{Duration, Instant},
};

use super::QueueError;
use compositor_render::{AudioSamplesBatch, AudioSamplesSet, InputId};

#[derive(Debug)]
pub struct AudioQueue {
    /// Enqueued sample batches per output
    input_queues: HashMap<InputId, VecDeque<AudioSamplesBatch>>,
    /// Stream added timestamp offset in relation to clock start
    timestamp_offsets: HashMap<InputId, Duration>,
    /// Stream starting pts offsets (starting pts doesn't have to be 0)
    start_pts: HashMap<InputId, Duration>,
}

impl AudioQueue {
    pub fn new() -> Self {
        AudioQueue {
            input_queues: HashMap::new(),
            timestamp_offsets: HashMap::new(),
            start_pts: HashMap::new(),
        }
    }

    pub fn add_input(&mut self, input_id: InputId) {
        self.input_queues.insert(input_id, VecDeque::new());
    }

    pub fn remove_input(&mut self, input_id: &InputId) {
        self.input_queues.remove(input_id);
    }

    pub fn enqueue_samples(
        &mut self,
        input_id: InputId,
        mut samples_batch: AudioSamplesBatch,
        clock_start: Instant,
    ) -> Result<(), QueueError> {
        let Some(input_queue) = self.input_queues.get_mut(&input_id) else {
            return Err(QueueError::UnknownInputId(input_id));
        };
        let offset = *self
            .timestamp_offsets
            .entry(input_id.clone())
            .or_insert_with(|| clock_start.elapsed());
        let start_pts = *self
            .start_pts
            .entry(input_id)
            .or_insert_with(|| samples_batch.pts);

        samples_batch.pts += offset;
        samples_batch.pts -= start_pts;

        input_queue.push_back(samples_batch);

        Ok(())
    }

    pub fn pop_samples_set(&mut self, pts: Duration, length: Duration) -> AudioSamplesSet {
        // Checks if any samples in batch are in [pts, pts + length] interval
        let batch_in_range =
            |batch: &AudioSamplesBatch| batch.pts < pts + length && batch.end() >= pts;
        let samples = self
            .input_queues
            .iter()
            .map(|(input_id, input_queue)| {
                let input_samples = input_queue
                    .iter()
                    .filter(|batch| batch_in_range(batch))
                    .cloned()
                    .collect::<Vec<AudioSamplesBatch>>();
                (input_id.clone(), input_samples)
            })
            .collect();

        self.drop_old_samples(pts + length);
        AudioSamplesSet {
            samples,
            pts,
            length,
        }
    }

    pub fn drop_old_samples(&mut self, up_to_pts: Duration) {
        for input_queue in self.input_queues.values_mut() {
            while input_queue
                .front()
                .map_or(false, |batch| batch.end() < up_to_pts)
            {
                input_queue.pop_front();
            }
        }
    }

    pub fn did_receive_samples(&self, input_id: &InputId) -> bool {
        self.timestamp_offsets.contains_key(input_id)
    }
}
