use std::{collections::HashMap, sync::Arc};
use thiserror::Error;

use super::{FramesBatch, InputID, MockFrame, PTS};

#[derive(Error, Debug)]
pub enum QueueError {
    #[error("the input id `{0}` is unknown")]
    UnknownInputID(InputID),
}

pub struct InternalQueue {
    // frames are PTS ordered
    inputs_queues: HashMap<InputID, Vec<Arc<MockFrame>>>,
}

impl InternalQueue {
    pub fn new() -> Self {
        InternalQueue {
            inputs_queues: HashMap::new(),
        }
    }

    pub fn add_input(&mut self, input_id: InputID) {
        self.inputs_queues.insert(input_id, Vec::new());
    }

    pub fn remove_input(&mut self, input_id: InputID) {
        self.inputs_queues.remove(&input_id);
    }

    pub fn enqueue_frame(&mut self, input_id: InputID, frame: MockFrame) -> Result<(), QueueError> {
        match self.inputs_queues.get_mut(&input_id) {
            Some(input_queue) => {
                input_queue.push(Arc::new(frame));
                Ok(())
            }
            None => Err(QueueError::UnknownInputID(input_id)),
        }
    }

    pub fn get_frames_batch(&mut self, buffer_pts: PTS) -> FramesBatch {
        let mut frames_batch = FramesBatch::new(buffer_pts);

        for (input_id, input_queue) in &self.inputs_queues {
            if let Some(nearest_frame) = input_queue.first() {
                frames_batch.insert_frame(*input_id, nearest_frame.clone());
            }
        }

        frames_batch
    }

    pub fn check_all_inputs_ready(&self, buffer_pts: PTS) -> bool {
        for input_queue in self.inputs_queues.values() {
            match input_queue.first() {
                Some(first_frame) => {
                    if first_frame.pts < buffer_pts && input_queue.last().unwrap().pts < buffer_pts
                    {
                        return false;
                    }
                }
                None => {
                    return false;
                }
            }
        }
        true
    }

    pub fn drop_pad_useless_frames(
        &mut self,
        input_id: InputID,
        next_buffer_pts: PTS,
    ) -> Result<(), QueueError> {
        let input_queue = self
            .inputs_queues
            .get_mut(&input_id)
            .ok_or(QueueError::UnknownInputID(input_id))?;

        if let Some(first_frame) = input_queue.first() {
            let mut best_diff_frame_index = 0;
            let mut best_diff_frame_pts = first_frame.pts;
            for (index, frame) in input_queue.iter().enumerate() {
                if frame.pts.abs_diff(next_buffer_pts)
                    <= best_diff_frame_pts.abs_diff(next_buffer_pts)
                {
                    best_diff_frame_index = index;
                    best_diff_frame_pts = frame.pts;
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

    pub fn drop_useless_frames(&mut self, next_buffer_pts: PTS) {
        let input_ids: Vec<InputID> = self.inputs_queues.keys().cloned().collect();
        for input_id in input_ids {
            self.drop_pad_useless_frames(input_id, next_buffer_pts)
                .unwrap();
        }
    }
}
