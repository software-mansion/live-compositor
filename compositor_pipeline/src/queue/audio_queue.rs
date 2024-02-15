use std::{
    collections::{HashMap, VecDeque},
    time::{Duration, Instant},
    vec,
};

use super::{audio_queue_thread::AudioQueueStartEvent, utils::InputState, InputOptions};
use compositor_render::{AudioSamplesBatch, AudioSamplesSet, InputId};
use crossbeam_channel::{Receiver, Sender, TryRecvError};
use log::error;

#[derive(Debug)]
pub struct AudioQueue {
    inputs: HashMap<InputId, AudioQueueInput>,
    start_sender: Option<Sender<AudioQueueStartEvent>>,
}

impl AudioQueue {
    pub fn new(start_sender: Sender<AudioQueueStartEvent>) -> Self {
        AudioQueue {
            inputs: HashMap::new(),
            start_sender: Some(start_sender),
        }
    }

    pub fn add_input(
        &mut self,
        input_id: &InputId,
        receiver: Receiver<AudioSamplesBatch>,
        opts: InputOptions,
    ) {
        self.inputs.insert(
            input_id.clone(),
            AudioQueueInput {
                queue: VecDeque::new(),
                receiver,
                input_state: InputState::WaitingForStart,
                required: opts.required,
                offset: opts.offset,
            },
        );
    }

    pub fn remove_input(&mut self, input_id: &InputId) {
        self.inputs.remove(input_id);
    }

    /// Checks if all inputs are ready to produce frames for specific PTS value (if all inputs have
    /// frames closest to buffer_pts).
    pub(super) fn check_all_inputs_ready_for_pts(
        &mut self,
        pts_range: (Duration, Duration),
        queue_start: Instant,
    ) -> bool {
        self.inputs
            .values_mut()
            .all(|input| input.check_ready_for_pts(pts_range, queue_start))
    }

    /// Checks if any of the required input stream have an offset that would
    /// require the stream to be used for PTS=`next_buffer_pts`
    pub(super) fn has_required_inputs_for_pts(
        &mut self,
        queue_pts: Duration,
        queue_start: Instant,
    ) -> bool {
        return self.inputs.values_mut().any(|input| {
            input.required
                && input
                    .input_pts_from_queue_pts(queue_pts, queue_start)
                    .is_some()
        });
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

    pub fn take_start_sender(&mut self) -> Option<Sender<AudioQueueStartEvent>> {
        std::mem::take(&mut self.start_sender)
    }
}

#[derive(Debug)]
struct AudioQueueInput {
    /// Samples/batches are PTS ordered where PTS=0 represents beginning of the stream.
    queue: VecDeque<AudioSamplesBatch>,
    /// Samples from the channel might have any PTS, they need to be processed before
    /// adding them to the `queue`.
    receiver: Receiver<AudioSamplesBatch>,
    /// Initial buffering + resets PTS to values starting with 0. All
    /// frames from receiver should be processed by this element.
    input_state: InputState<AudioSamplesBatch>,
    /// If stream is required the queue should wait for frames. For optional
    /// inputs a queue will wait only as long as a buffer allows.
    required: bool,
    /// Offset of the stream relative to the start. If set to `None`
    /// offset will be resolved automatically on the stream start.
    offset: Option<Duration>,
}

impl AudioQueueInput {
    /// Get batches that have samples in range `range` and remove them from the queue.
    /// Batches that are partially in range will still be returned, but they won't be
    /// removed from the queue.
    fn pop_samples(
        &mut self,
        pts_range: (Duration, Duration),
        queue_start: Instant,
    ) -> Vec<AudioSamplesBatch> {
        // range in queue pts time frame
        let (start_pts, end_pts) = pts_range;

        // range in input pts time frame
        let (Some(start_pts), Some(end_pts)) = (
            self.input_pts_from_queue_pts(start_pts, queue_start),
            self.input_pts_from_queue_pts(end_pts, queue_start),
        ) else {
            error!("This should not happen. Unable to calculate PTS in input time frame.");
            return vec![];
        };
        let Some(input_start_time) = self.input_start_time() else {
            error!("This should not happen. Unable to resolve input start time.");
            return vec![];
        };

        let popped_samples = self
            .queue
            .iter()
            // start_pts and end_pts are already in units of this input
            .filter(|batch| batch.start_pts < end_pts || batch.end_pts() > start_pts)
            .cloned()
            .map(|mut batch| {
                match self.offset {
                    Some(offset) => {
                        batch.start_pts += offset;
                    }
                    None => {
                        batch.start_pts =
                            (input_start_time + batch.start_pts).duration_since(queue_start);
                    }
                }
                batch
            })
            .collect::<Vec<AudioSamplesBatch>>();

        self.drop_old_samples(pts_range.1, queue_start);

        popped_samples
    }

    fn check_ready_for_pts(
        &mut self,
        pts_range: (Duration, Duration),
        queue_start: Instant,
    ) -> bool {
        // range in queue pts time frame
        let end_pts = pts_range.0;

        // range in input pts time frame
        let Some(end_pts) = self.input_pts_from_queue_pts(end_pts, queue_start) else {
            return match self.offset {
                Some(offset) => {
                    // If stream should start latter than `end_pts`, then it's fine
                    // to consider it ready, because we will not use samples for that PTS
                    // regardless if they are there or not.
                    offset > end_pts
                }
                None => {
                    // It represent stream that still buffering. We now that frames
                    // from this input will not be used for this batch, so it is fine
                    // to consider this "ready".
                    true
                }
            };
        };

        fn has_are_samples_for_pts_range(
            queue: &VecDeque<AudioSamplesBatch>,
            range_end_pts: Duration,
        ) -> bool {
            match queue.back() {
                Some(batch) => batch.end_pts() > range_end_pts,
                None => false,
            }
        }

        while !has_are_samples_for_pts_range(&self.queue, end_pts) {
            if self.try_enqueue_samples().is_err() {
                return false;
            }
        }
        true
    }

    /// Drop all batches older than `pts`. Entire batch (all samples inside) has to be older.
    fn drop_old_samples(&mut self, queue_pts: Duration, queue_start: Instant) {
        let Some(pts) = self.input_pts_from_queue_pts(queue_pts, queue_start) else {
            // before first sample so nothing to drop
            return;
        };
        while self
            .queue
            .front()
            .map_or(false, |batch| batch.end_pts() < pts)
        {
            self.queue.pop_front();
        }
    }

    /// Calculate input pts based on queue pts and queue start time. It can trigger
    /// enqueue internally.
    ///
    /// Returns None if:
    /// - Input is not ready and offset is unknown
    /// - If offset is negative (PTS refers to moment from before stream start)
    fn input_pts_from_queue_pts(
        &mut self,
        queue_pts: Duration,
        queue_start_time: Instant,
    ) -> Option<Duration> {
        match self.offset {
            Some(offset) => queue_pts.checked_sub(offset),
            None => match self.input_start_time() {
                Some(input_start_time) => {
                    (queue_start_time + queue_pts).checked_duration_since(input_start_time)
                }
                None => None,
            },
        }
    }

    /// Evaluate start time of this input. Start time represents an instant of time
    /// when input switched from buffering state to ready.
    fn input_start_time(&mut self) -> Option<Instant> {
        loop {
            if let InputState::Ready { start_time, .. } = self.input_state {
                return Some(start_time);
            }

            if self.try_enqueue_samples().is_err() {
                return None;
            }
        }
    }

    fn try_enqueue_samples(&mut self) -> Result<(), TryRecvError> {
        let samples_batch = self.receiver.try_recv()?;
        let original_pts = samples_batch.start_pts;

        let mut batches = self
            .input_state
            .process_new_chunk(samples_batch, original_pts);
        self.queue.append(&mut batches);

        Ok(())
    }
}
