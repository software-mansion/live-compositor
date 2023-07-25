use compositor_common::Frame;
use compositor_common::{Framerate, InputId};
use compositor_render::input_frames::InputFrames;
use std::time::Duration;
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum QueueError {
    #[error("the input id `{0}` is unknown")]
    UnknownInputId(InputId),
}

pub struct InternalQueue {
    // frames are PTS ordered. PTS include timestamps offsets
    inputs_queues: HashMap<InputId, Vec<Arc<Frame>>>,
    timestamp_offsets: HashMap<InputId, Duration>,
    output_framerate: Framerate,
    pub send_batches_counter: u32,
}

impl InternalQueue {
    pub fn new(output_framerate: Framerate) -> Self {
        InternalQueue {
            inputs_queues: HashMap::new(),
            timestamp_offsets: HashMap::new(),
            output_framerate,
            send_batches_counter: 0,
        }
    }

    pub fn add_input(&mut self, input_id: InputId) {
        self.inputs_queues.insert(input_id, Vec::new());
    }

    pub fn remove_input(&mut self, input_id: InputId) {
        self.inputs_queues.remove(&input_id);
        self.timestamp_offsets.remove(&input_id);
    }

    pub fn enqueue_frame(&mut self, input_id: InputId, mut frame: Frame) -> Result<(), QueueError> {
        let next_output_buffer_pts = self.get_next_output_buffer_pts();

        match self.inputs_queues.get_mut(&input_id) {
            Some(input_queue) => {
                let offset = *self
                    .timestamp_offsets
                    .entry(input_id)
                    .or_insert_with(|| next_output_buffer_pts.saturating_sub(frame.pts));

                frame.pts += offset;

                input_queue.push(Arc::new(frame));
                Ok(())
            }
            None => Err(QueueError::UnknownInputId(input_id)),
        }
    }

    /// Pops frames closest to buffer pts.
    /// Implementation assumes that "useless frames" are already dropped
    /// by [`drop_useless_frames`] or [`drop_pad_useless_frames`]
    pub fn get_frames_batch(&mut self, buffer_pts: Duration) -> InputFrames {
        let mut frames_batch = InputFrames::new(buffer_pts);

        for (input_id, input_queue) in &self.inputs_queues {
            if let Some(nearest_frame) = input_queue.first() {
                frames_batch.insert_frame(*input_id, nearest_frame.clone());
            }
        }
        self.send_batches_counter += 1;

        frames_batch
    }

    // Checks if all inputs have frames closest to buffer_pts.
    // Every input queue should have frame with larger or equal pts than buffer pts.
    // We assume, that queue receives frames with monotonically increasing timestamps,
    // so when all inputs queues have frame with pts larger or equal than buffer timestamp,
    // queue won't receive frame with pts "closer" to buffer pts.
    // In other cases, queue might receive frame "closer" to buffer pts in future.
    pub fn check_all_inputs_ready(&self, buffer_pts: Duration) -> bool {
        self.inputs_queues
            .values()
            .all(|input_queue| match input_queue.last() {
                Some(last_frame) => last_frame.pts >= buffer_pts,
                None => false,
            })
    }

    // Drops frames that won't be used anymore by the VideoCompositor from single input.
    // Since VideoCompositor received and enqueues frames with monotonically increasing pts,
    // all frames in `input queue` placed before frame with best pts difference to next buffer pts
    // won't be used anymore. This queue always wants to minimize pts diff between frame pts and
    // send buffer pts. We can be certain that frame X from input I
    // won't be the closest one to any next buffers if and only if
    // queue received frame Y with lower pts diff to next buffer and larger PTS than frame X.
    pub fn drop_input_useless_frames(&mut self, input_id: InputId) -> Result<(), QueueError> {
        let next_output_buffer_nanos = self.get_next_output_buffer_pts().as_nanos();

        let input_queue = self
            .inputs_queues
            .get_mut(&input_id)
            .ok_or(QueueError::UnknownInputId(input_id))?;

        if input_queue.first().is_some() {
            let best_diff_frame_index = input_queue
                .iter()
                .enumerate()
                .min_by_key(|(_index, frame)| {
                    frame.pts.as_nanos().abs_diff(next_output_buffer_nanos)
                })
                .map_or(0, |(index, _frame)| index);

            input_queue.drain(0..best_diff_frame_index);
        }

        Ok(())
    }

    pub fn drop_useless_frames(&mut self) {
        let input_ids: Vec<InputId> = self.inputs_queues.keys().cloned().collect();
        for input_id in input_ids {
            self.drop_input_useless_frames(input_id).unwrap();
        }
    }

    pub fn get_next_output_buffer_pts(&self) -> Duration {
        Duration::from_secs_f64(self.send_batches_counter as f64 / self.output_framerate.0 as f64)
    }
}
