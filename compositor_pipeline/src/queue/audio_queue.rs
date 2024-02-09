use std::{
    collections::{HashMap, VecDeque},
    time::{Duration, Instant},
};

use super::utils::InputState;
use compositor_render::{AudioSamplesBatch, AudioSamplesSet, InputId};
use crossbeam_channel::{Receiver, TryRecvError};

#[derive(Debug)]
pub struct AudioQueue {
    inputs: HashMap<InputId, AudioQueueInput>,
}

impl AudioQueue {
    pub fn new() -> Self {
        AudioQueue {
            inputs: HashMap::new(),
        }
    }

    pub fn add_input(&mut self, input_id: &InputId, receiver: Receiver<AudioSamplesBatch>) {
        self.inputs.insert(
            input_id.clone(),
            AudioQueueInput {
                queue: VecDeque::new(),
                receiver,
                input_state: InputState::WaitingForStart,
            },
        );
    }

    pub fn remove_input(&mut self, input_id: &InputId) {
        self.inputs.remove(input_id);
    }

    pub fn pop_samples_set(
        &mut self,
        range: (Duration, Duration),
        clock_start: Instant,
    ) -> AudioSamplesSet {
        let (start_pts, end_pts) = range;
        let samples = self
            .inputs
            .iter_mut()
            .map(|(input_id, input)| (input_id.clone(), input.pop_samples(range, clock_start)))
            .collect();

        AudioSamplesSet {
            samples,
            start_pts,
            length: end_pts.saturating_sub(start_pts),
        }
    }
}

#[derive(Debug)]
struct AudioQueueInput {
    /// Frames are PTS ordered. PTS include timestamps offsets
    queue: VecDeque<AudioSamplesBatch>,
    /// Frames from the channel might have any PTS. When enqueuing
    /// they need to be recalculated relative to `Queue:clock_start`.
    receiver: Receiver<AudioSamplesBatch>,

    /// Controls input initialization, buffering, and stores information
    /// about input offset.
    input_state: InputState<AudioSamplesBatch>,
}

impl AudioQueueInput {
    /// Get batches that have samples in range `range` and remove them from the queue.
    /// Batches that are partially in range will still be returned, but they won't be
    /// removed from the queue.
    fn pop_samples(
        &mut self,
        range: (Duration, Duration),
        clock_start: Instant,
    ) -> Vec<AudioSamplesBatch> {
        let (start_pts, end_pts) = range;

        fn is_ready(queue: &VecDeque<AudioSamplesBatch>, range_end_pts: Duration) -> bool {
            match queue.back() {
                Some(batch) => batch.start_pts > range_end_pts,
                None => false,
            }
        }

        while !is_ready(&self.queue, end_pts) {
            if self.try_enqueue_samples(clock_start).is_err() {
                break;
            }
        }

        let poped_samples = self
            .queue
            .iter()
            .filter(|batch| batch.start_pts < end_pts && batch.end_pts() > start_pts)
            .cloned()
            .collect::<Vec<AudioSamplesBatch>>();
        self.drop_old_samples(end_pts);

        poped_samples
    }

    fn try_enqueue_samples(&mut self, clock_start: Instant) -> Result<(), TryRecvError> {
        let samples_batch = self.receiver.try_recv()?;
        let original_pts = samples_batch.start_pts;

        let mut batches =
            self.input_state
                .process_new_chunk(samples_batch, original_pts, clock_start);
        self.queue.append(&mut batches);

        Ok(())
    }

    /// Drop all batches older than `pts`. Entire batch (all samples inside) has to be older.
    fn drop_old_samples(&mut self, pts: Duration) {
        while self
            .queue
            .front()
            .map_or(false, |batch| batch.end_pts() < pts)
        {
            self.queue.pop_front();
        }
    }
}
