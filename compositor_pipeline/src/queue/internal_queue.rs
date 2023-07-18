use anyhow::anyhow;
use compositor_common::Frame;
use compositor_common::frame::{InputID, Framerate, Pts, FramesBatch};
use std::collections::hash_map::Entry::Vacant;
use std::time::Duration;
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;


#[derive(Error, Debug)]
pub enum QueueError {
    #[error("the input id `{0}` is unknown")]
    UnknownInputID(InputID),
}

pub struct InternalQueue {
    // frames are PTS ordered. PTS include timestamps offsets
    inputs_queues: HashMap<InputID, Vec<Arc<Frame>>>,
    timestamp_offsets: HashMap<InputID, Pts>,
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

    pub fn add_input(&mut self, input_id: InputID) {
        self.inputs_queues.insert(input_id, Vec::new());
    }

    pub fn remove_input(&mut self, input_id: InputID) {
        self.inputs_queues.remove(&input_id);
        self.timestamp_offsets.remove(&input_id);
    }

    pub fn enqueue_frame(&mut self, input_id: InputID, mut frame: Frame) -> Result<(), QueueError> {
        match self.inputs_queues.get_mut(&input_id) {
            Some(input_queue) => {
                if let Vacant(e) = self.timestamp_offsets.entry(input_id) {
                    e.insert(frame.pts);
                }

                frame.pts += *self
                    .timestamp_offsets
                    .get(&input_id)
                    .ok_or_else(|| anyhow!("Timestamp offset unregistered for pad {input_id}"))
                    .unwrap();

                input_queue.push(Arc::new(frame));
                Ok(())
            }
            None => Err(QueueError::UnknownInputID(input_id)),
        }
    }

    /// Pops frames closest to buffer pts.
    /// Implementation assumes that "useless frames" are already dropped
    /// by [`drop_useless_frames`] or [`drop_pad_useless_frames`]
    pub fn get_frames_batch(&mut self, buffer_pts: Pts) -> FramesBatch {
        let mut frames_batch = FramesBatch::new(buffer_pts);

        for (input_id, input_queue) in &self.inputs_queues {
            if let Some(nearest_frame) = input_queue.first() {
                frames_batch.insert_frame(*input_id, nearest_frame.clone());
            }
        }
        self.send_batches_counter += 1;

        frames_batch
    }

    /// Checks if all inputs have frames closest to buffer_pts.
    // Every input queue should have frame with larger or equal pts than buffer pts.
    // We assume, that queue receives frames with monotonically increasing timestamps,
    // so when all inputs queues have frame with pts larger or equal than buffer timestamp,
    // queue won't receive frame with pts "closer" to buffer pts.
    // In other cases, queue might receive frame "closer" to buffer pts in future.
    pub fn check_all_inputs_ready(&self, buffer_pts: Pts) -> bool {
        self.inputs_queues
            .values()
            .all(|input_queue| match input_queue.last() {
                Some(last_frame) => last_frame.pts >= buffer_pts,
                None => false,
            })
    }

    pub fn drop_pad_useless_frames(&mut self, input_id: InputID) -> Result<(), QueueError> {
        let next_output_buffer_nanos = self.get_next_output_buffer_pts().as_nanos();

        let input_queue = self
            .inputs_queues
            .get_mut(&input_id)
            .ok_or(QueueError::UnknownInputID(input_id))?;

        if let Some(first_frame) = input_queue.first() {
            let mut best_diff_frame_index = 0;
            let mut best_diff_nanos = first_frame
                .pts
                .as_nanos()
                .abs_diff(next_output_buffer_nanos);

            for (index, frame) in input_queue.iter().enumerate() {
                let diff = frame.pts.as_nanos().abs_diff(next_output_buffer_nanos);
                if diff <= best_diff_nanos {
                    best_diff_frame_index = index;
                    best_diff_nanos = diff;
                } else {
                    break;
                }
            }

            if best_diff_frame_index > 0 {
                input_queue.drain(0..best_diff_frame_index);
            }
        }

        Ok(())
    }

    pub fn drop_useless_frames(&mut self) {
        let input_ids: Vec<InputID> = self.inputs_queues.keys().cloned().collect();
        for input_id in input_ids {
            self.drop_pad_useless_frames(input_id).unwrap();
        }
    }

    pub fn get_next_output_buffer_pts(&self) -> Pts {
        Duration::from_secs_f64(self.send_batches_counter as f64 / self.output_framerate.0 as f64)
    }
}
