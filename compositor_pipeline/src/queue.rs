mod internal_queue;

use std::{
    sync::{Arc, Mutex},
    thread::{self, sleep},
    time::{Duration, Instant},
};

use compositor_common::{scene::InputId, Frame, Framerate};
use compositor_render::frame_set::FrameSet;
use crossbeam_channel::{tick, unbounded, Receiver, Sender};
use thiserror::Error;

use self::internal_queue::InternalQueue;

#[derive(Error, Debug)]
pub enum QueueError {
    #[error("the input id `{:#?}` is unknown", 0)]
    UnknownInputId(InputId),
}

const DEFAULT_BUFFER_DURATION: Duration = Duration::from_millis(10);

pub struct Queue {
    internal_queue: Arc<Mutex<InternalQueue>>,
    check_queue_channel: (Sender<()>, Receiver<()>),
    output_framerate: Framerate,
    buffer_duration: Duration,
}

impl Queue {
    pub fn new(output_framerate: Framerate) -> Self {
        Queue {
            internal_queue: Arc::new(Mutex::new(InternalQueue::new(output_framerate))),
            check_queue_channel: unbounded(),
            output_framerate,
            buffer_duration: DEFAULT_BUFFER_DURATION,
        }
    }

    pub fn add_input(&self, input_id: InputId) {
        let mut internal_queue = self.internal_queue.lock().unwrap();
        internal_queue.add_input(input_id);
    }

    #[allow(dead_code)]
    pub fn remove_input(&self, input_id: InputId) {
        let mut internal_queue = self.internal_queue.lock().unwrap();
        // TODO: gracefully remove input - wait until last enqueued frame PTS is smaller than output PTS
        internal_queue.remove_input(input_id);
    }

    pub fn start(&self, sender: Sender<FrameSet<InputId>>) {
        let (check_queue_sender, check_queue_receiver) = self.check_queue_channel.clone();
        let internal_queue = self.internal_queue.clone();
        let tick_duration = self.output_framerate.get_interval_duration();
        let buffer_duration = self.buffer_duration;

        thread::spawn(move || {
            // Wait for first frame
            check_queue_receiver.recv().unwrap();
            sleep(buffer_duration);

            Self::start_ticker(tick_duration, check_queue_sender);

            let start = Instant::now();
            loop {
                check_queue_receiver.recv().unwrap();

                let mut internal_queue = internal_queue.lock().unwrap();
                let buffer_pts = internal_queue.get_next_output_buffer_pts();

                if start.elapsed() + buffer_duration > buffer_pts
                    || internal_queue.check_all_inputs_ready(buffer_pts)
                {
                    let frames_batch = internal_queue.get_frames_batch(buffer_pts);
                    sender.send(frames_batch).unwrap();
                    internal_queue.drop_useless_frames();
                }
            }
        });
    }

    pub fn enqueue_frame(&self, input_id: InputId, frame: Frame) -> Result<(), QueueError> {
        let mut internal_queue = self.internal_queue.lock().unwrap();

        internal_queue.enqueue_frame(input_id.clone(), frame)?;
        internal_queue.drop_input_useless_frames(input_id)?;

        self.check_queue_channel.0.send(()).unwrap();

        Ok(())
    }

    fn start_ticker(tick_duration: Duration, check_queue_sender: Sender<()>) {
        thread::spawn(move || {
            let ticker = tick(tick_duration);
            check_queue_sender.send(()).unwrap();
            loop {
                ticker.recv().unwrap();
                check_queue_sender.send(()).unwrap();
            }
        });
    }
}
