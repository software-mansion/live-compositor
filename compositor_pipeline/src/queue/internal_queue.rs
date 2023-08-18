use compositor_common::scene::InputId;
use compositor_common::Frame;
use compositor_render::frame_set::FrameSet;

use std::collections::HashMap;
use std::time::Duration;
use std::time::Instant;

use super::QueueError;

pub struct InternalQueue {
    /// frames are PTS ordered. PTS include timestamps offsets
    inputs_queues: HashMap<InputId, Vec<Frame>>,
    inputs_listeners: HashMap<InputId, Vec<Box<dyn FnOnce() + Send>>>,
    /// offsets that normalize input pts to zero relative to the
    /// Queue:clock_start value.
    timestamp_offsets: HashMap<InputId, Duration>,
}

impl InternalQueue {
    pub fn new() -> Self {
        InternalQueue {
            inputs_queues: HashMap::new(),
            inputs_listeners: HashMap::new(),
            timestamp_offsets: HashMap::new(),
        }
    }

    pub fn add_input(&mut self, input_id: InputId) {
        self.inputs_queues.insert(input_id, Vec::new());
    }

    pub fn remove_input(&mut self, input_id: &InputId) {
        self.inputs_queues.remove(input_id);
        self.timestamp_offsets.remove(input_id);
    }

    pub fn did_receive_frame(&self, input_id: &InputId) -> bool {
        self.timestamp_offsets.get(input_id).is_some()
    }

    pub fn enqueue_frame(
        &mut self,
        input_id: InputId,
        mut frame: Frame,
        clock_start: Instant,
    ) -> Result<(), QueueError> {
        let Some(input_queue) = self.inputs_queues.get_mut(&input_id) else {
            return Err(QueueError::UnknownInputId(input_id))
        };

        let offset = *self
            .timestamp_offsets
            .entry(input_id)
            .or_insert_with(|| clock_start.elapsed().saturating_sub(frame.pts));

        // Modify frame pts to be at the time frame where PTS=0 represent clock_start
        frame.pts += offset;

        input_queue.push(frame);
        Ok(())
    }

    /// Gets frames closest to buffer pts.
    pub fn get_frames_batch(&mut self, buffer_pts: Duration) -> FrameSet<InputId> {
        for (_, input_queue) in self.inputs_queues.iter_mut() {
            Self::drop_old_input_frames(input_queue, buffer_pts);
        }

        let mut frames_batch = FrameSet::new(buffer_pts);
        for (input_id, input_queue) in &self.inputs_queues {
            if let Some(nearest_frame) = input_queue.first() {
                frames_batch
                    .frames
                    .insert(input_id.clone(), nearest_frame.clone());
            }
        }

        frames_batch
    }

    /// Checks if all inputs have frames closest to buffer_pts.
    ///
    /// Every input queue should have a frame with larger or equal pts than buffer pts.
    /// We assume that the queue receives frames with monotonically increasing timestamps,
    /// so when all inputs queues have frames with pts larger or equal than buffer timestamp,
    /// the queue won't receive frames with pts "closer" to buffer pts.
    /// When the queue hasn't received a frame with pts larger or equal than buffer timestamp on every
    /// input, queue might receive frame "closer" to buffer pts in the future on some input,
    /// therefore it should wait with buffer push until it receives those frames or until
    /// ticker enforces push from the queue.
    pub fn check_all_inputs_ready(&self, next_buffer_pts: Duration) -> bool {
        self.inputs_queues
            .values()
            .all(|input_queue| match input_queue.last() {
                Some(last_frame) => last_frame.pts >= next_buffer_pts,
                None => false,
            })
    }

    /// Drops frames that won't be used anymore by the VideoCompositor from a single input.
    ///
    /// Finds frame that is closest to the next_buffer_pts and removes everything older.
    /// Frames in queue have monotonically increasing pts, so we can just drop all the frames
    /// before the "closest" one.
    fn drop_old_input_frames(input_queue: &mut Vec<Frame>, next_buffer_pts: Duration) {
        let next_output_buffer_nanos = next_buffer_pts.as_nanos();
        let closest_diff_frame_index = input_queue
            .iter()
            .enumerate()
            .min_by_key(|(_index, frame)| frame.pts.as_nanos().abs_diff(next_output_buffer_nanos))
            .map(|(index, _frame)| index);

        if let Some(index) = closest_diff_frame_index {
            input_queue.drain(0..index);
        }
    }

    pub fn drop_old_frames_by_input_id(
        &mut self,
        input_id: &InputId,
        next_buffer_pts: Duration,
    ) -> Result<(), QueueError> {
        let input_queue = self
            .inputs_queues
            .get_mut(input_id)
            .ok_or_else(|| QueueError::UnknownInputId(input_id.clone()))?;

        Self::drop_old_input_frames(input_queue, next_buffer_pts);
        Ok(())
    }

    pub fn subscribe_input_listener(
        &mut self,
        input_id: InputId,
        callback: Box<dyn FnOnce() + Send>,
    ) {
        self.inputs_listeners
            .entry(input_id)
            .or_insert_with(Vec::new)
            .push(callback)
    }

    pub fn call_input_listeners(&mut self, input_id: &InputId) {
        let callbacks = self.inputs_listeners.remove(input_id).unwrap_or_default();
        for cb in callbacks.into_iter() {
            cb()
        }
    }
}
