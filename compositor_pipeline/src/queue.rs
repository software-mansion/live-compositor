mod internal_queue;
mod queue_thread;

use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use compositor_common::{error::ErrorStack, scene::InputId, Frame, Framerate};
use compositor_render::FrameSet;
use crossbeam_channel::{unbounded, Receiver, Sender};
use log::error;
use thiserror::Error;

use self::{internal_queue::InternalQueue, queue_thread::QueueThread};

#[derive(Error, Debug)]
pub enum QueueError {
    #[error("the input id `{:#?}` is unknown", 0)]
    UnknownInputId(InputId),
}

const DEFAULT_BUFFER_DURATION: Duration = Duration::from_millis(16 * 5); // about 5 frames at 60 fps

/// Queue is responsible for consuming frames from different inputs and producing
/// sets of frames from all inputs in a single batch.
///
/// PTS after frame is enqueued:
/// - PTS=0 should represent clock_start instant for each input.
/// - Queue is force pushing frames when time that represents the next PTS
///   is earlier than Instant.now(). We don't need any buffering because we
///   are blocking on the first frame of each input.
/// - Technically real_pts = frame.pts - buffer_duration, but because relative value
///   does not matter we don't need to take that into account.
pub struct Queue {
    internal_queue: Mutex<InternalQueue>,
    check_queue_channel: (Sender<()>, Receiver<()>),
    output_framerate: Framerate,

    /// - When new input is connected and sends the first frame we want to wait
    /// buffer_duration before sending first frame of that input.
    /// - When pipeline is started we want to start with a frame that was receive
    /// `buffer_duration` time ago
    buffer_duration: Duration,

    /// Base time that is used to synchronize PTS value of received frame to
    /// the same clock. When enqueueing the frame we are modifying it's PTS
    /// using (per input) offset calculated based on this clock.
    ///
    /// The end goal is that resulting PTS should be in a time frame where PTS
    /// is equivalent to clock_start time of real time.
    clock_start: Instant,
}

impl Queue {
    pub fn new(output_framerate: Framerate) -> Self {
        Queue {
            internal_queue: Mutex::new(InternalQueue::new()),
            check_queue_channel: unbounded(),
            output_framerate,
            buffer_duration: DEFAULT_BUFFER_DURATION,
            clock_start: Instant::now(),
        }
    }

    pub fn add_input(&self, input_id: InputId) {
        self.internal_queue.lock().unwrap().add_input(input_id);
    }

    pub fn remove_input(&self, input_id: &InputId) {
        self.internal_queue.lock().unwrap().remove_input(input_id);
    }

    pub fn start(self: &Arc<Self>, sender: Sender<FrameSet<InputId>>) {
        let queue = self.clone();
        let tick_duration = self.output_framerate.get_interval_duration();

        QueueThread::new(
            queue,
            sender,
            queue_thread::Options {
                tick_duration,
                buffer_duration: self.buffer_duration,
                output_framerate: self.output_framerate,
                clock_start: self.clock_start,
            },
        )
        .spawn();
    }

    pub fn enqueue_frame(&self, input_id: InputId, frame: Frame) -> Result<(), QueueError> {
        let is_first_frame_for_input = !self
            .internal_queue
            .lock()
            .unwrap()
            .did_receive_frame(&input_id);
        if is_first_frame_for_input {
            // Sleep here ensures that we will buffer `self.buffer_duration` on each input.
            // It also makes calculation easier because PTS of frames will be already offset
            // by a correct value.
            thread::sleep(self.buffer_duration);
        }

        let mut internal_queue = self.internal_queue.lock().unwrap();

        internal_queue.enqueue_frame(input_id.clone(), frame, self.clock_start)?;

        // We don't know when pipeline is started, so we can't resolve real_next_pts,
        // but we can remove frames based on estimated PTS. This only works if queue
        // is able to push frames in real time and is never behind more than one frame.
        let framerate_tick = Duration::from_secs_f64(1.0 / self.output_framerate.0 as f64);
        let estimated_pts = self.clock_start.elapsed() - framerate_tick;
        if let Err(err) = internal_queue.drop_old_frames_by_input_id(&input_id, estimated_pts) {
            error!(
                "Failed to drop frames on input {}:\n{}",
                input_id,
                ErrorStack::new(&err).into_string()
            )
        }

        self.check_queue_channel.0.send(()).unwrap();

        Ok(())
    }

    pub fn subscribe_input_listener(&self, input_id: InputId, callback: Box<dyn FnOnce() + Send>) {
        self.internal_queue
            .lock()
            .unwrap()
            .subscribe_input_listener(input_id, callback)
    }
}
