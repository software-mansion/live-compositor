mod internal_queue;
mod queue_thread;

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use compositor_common::{scene::InputId, Frame, Framerate};
use compositor_render::frame_set::FrameSet;
use crossbeam_channel::{unbounded, Receiver, Sender};
use thiserror::Error;

use self::{internal_queue::InternalQueue, queue_thread::QueueThread};

#[derive(Error, Debug)]
pub enum QueueError {
    #[error("the input id `{:#?}` is unknown", 0)]
    UnknownInputId(InputId),
}

const DEFAULT_BUFFER_DURATION: Duration = Duration::from_millis(10);

pub struct Queue {
    internal_queue: Mutex<InternalQueue>,
    check_queue_channel: (Sender<()>, Receiver<()>),
    output_framerate: Framerate,
    buffer_duration: Duration,
}

impl Queue {
    pub fn new(output_framerate: Framerate) -> Self {
        Queue {
            internal_queue: Mutex::new(InternalQueue::new(output_framerate)),
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
    pub fn remove_input(&self, input_id: &InputId) {
        let mut internal_queue = self.internal_queue.lock().unwrap();
        // TODO: gracefully remove input - wait until last enqueued frame PTS is smaller than output PTS
        internal_queue.remove_input(input_id);
    }

    pub fn start(self: &Arc<Self>, sender: Sender<FrameSet<InputId>>) {
        let queue = self.clone();
        let tick_duration = self.output_framerate.get_interval_duration();
        let buffer_duration = self.buffer_duration;

        QueueThread::new(
            queue,
            sender,
            queue_thread::Options {
                buffer_duration,
                tick_duration,
            },
        )
        .spawn();
    }

    pub fn enqueue_frame(&self, input_id: InputId, frame: Frame) -> Result<(), QueueError> {
        let mut internal_queue = self.internal_queue.lock().unwrap();

        internal_queue.enqueue_frame(input_id.clone(), frame)?;
        internal_queue.drop_input_useless_frames(input_id)?;

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
