use compositor_render::Frame;
use compositor_render::FrameSet;
use compositor_render::InputId;
use crossbeam_channel::Receiver;
use crossbeam_channel::TryRecvError;

use std::collections::HashMap;
use std::collections::VecDeque;
use std::mem;
use std::time::Duration;
use std::time::Instant;

use super::utils::InputState;

pub struct VideoQueue {
    inputs: HashMap<InputId, VideoQueueInput>,
}

impl VideoQueue {
    pub fn new() -> Self {
        VideoQueue {
            inputs: HashMap::new(),
        }
    }

    pub fn add_input(&mut self, input_id: &InputId, receiver: Receiver<Frame>) {
        self.inputs.insert(
            input_id.clone(),
            VideoQueueInput {
                queue: VecDeque::new(),
                receiver,
                listeners: vec![],
                input_state: InputState::WaitingForStart,
            },
        );
    }

    pub fn remove_input(&mut self, input_id: &InputId) {
        self.inputs.remove(input_id);
    }

    /// Gets frames closest to buffer pts. It does not check whether input is ready
    /// or not.
    pub fn get_frames_batch(&mut self, buffer_pts: Duration) -> FrameSet<InputId> {
        let frames = self
            .inputs
            .iter_mut()
            .filter_map(|(input_id, input)| {
                input
                    .get_frame(buffer_pts)
                    .map(|frame| (input_id.clone(), frame))
            })
            .collect();

        FrameSet {
            frames,
            pts: buffer_pts,
        }
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
    pub(super) fn check_all_inputs_ready(
        &mut self,
        next_buffer_pts: Duration,
        clock_start: Instant,
    ) -> bool {
        self.inputs
            .values_mut()
            .all(|input| input.check_if_ready(next_buffer_pts, clock_start))
    }

    pub fn subscribe_input_listener(
        &mut self,
        input_id: &InputId,
        callback: Box<dyn FnOnce() + Send>,
    ) {
        if let Some(input) = self.inputs.get_mut(input_id) {
            input.listeners.push(callback)
        }
    }

    pub fn call_input_listeners(&mut self, input_id: &InputId) {
        if let Some(input) = self.inputs.get_mut(input_id) {
            for cb in mem::take(&mut input.listeners).into_iter() {
                cb()
            }
        }
    }
}

pub struct VideoQueueInput {
    /// Frames are PTS ordered. PTS include timestamps offsets
    queue: VecDeque<Frame>,
    /// Frames from the channel might have any PTS. When enqueuing
    /// they need to be recalculated relative to `Queue:clock_start`.
    receiver: Receiver<Frame>,
    listeners: Vec<Box<dyn FnOnce() + Send>>,

    /// Controls input initialization, buffering, and stores information
    /// about input offset.
    input_state: InputState<Frame>,
}

impl VideoQueueInput {
    fn get_frame(&mut self, buffer_pts: Duration) -> Option<Frame> {
        self.drop_old_frames(buffer_pts);
        self.queue.front().cloned()
    }

    fn check_if_ready(&mut self, next_buffer_pts: Duration, clock_start: Instant) -> bool {
        fn is_ready(queue: &VecDeque<Frame>, next_buffer_pts: Duration) -> bool {
            match queue.back() {
                Some(last_frame) => last_frame.pts >= next_buffer_pts,
                None => false,
            }
        }

        while !is_ready(&self.queue, next_buffer_pts) {
            if self.try_enqueue_frame(clock_start).is_err() {
                return false;
            }
        }
        true
    }

    fn try_enqueue_frame(&mut self, clock_start: Instant) -> Result<(), TryRecvError> {
        let frame = self.receiver.try_recv()?;
        let original_pts = frame.pts;

        let mut frames = self
            .input_state
            .process_new_chunk(frame, original_pts, clock_start);
        self.queue.append(&mut frames);

        Ok(())
    }

    /// Drops frames that won't be used anymore by the VideoCompositor from a single input.
    ///
    /// Finds frame that is closest to the next_buffer_pts and removes everything older.
    /// Frames in queue have monotonically increasing pts, so we can just drop all the frames
    /// before the "closest" one.
    fn drop_old_frames(&mut self, next_buffer_pts: Duration) {
        let next_output_buffer_nanos = next_buffer_pts.as_nanos();
        let closest_diff_frame_index = self
            .queue
            .iter()
            .enumerate()
            .min_by_key(|(_index, frame)| frame.pts.as_nanos().abs_diff(next_output_buffer_nanos))
            .map(|(index, _frame)| index);

        if let Some(index) = closest_diff_frame_index {
            self.queue.drain(0..index);
        }
    }
}
