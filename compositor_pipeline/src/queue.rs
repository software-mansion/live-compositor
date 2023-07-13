mod internal_queue;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread::{self, sleep},
    time::{Duration, Instant},
};

use compositor_common::Frame;
use crossbeam_channel::{tick, unbounded, Receiver, Sender};

use self::internal_queue::{InternalQueue, QueueError};

pub type InputID = u32;

/// nanoseconds
type Pts = i64;

/// TODO: This should be a rational.
#[derive(Debug, Clone, Copy)]
pub struct Framerate(pub u32);

impl Framerate {
    pub fn get_interval_duration(self) -> Duration {
        Duration::from_nanos((1_000_000_000 / self.0).into())
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct FramesBatch {
    pub frames: HashMap<InputID, Arc<Frame>>,
    pub pts: Pts,
}

impl FramesBatch {
    pub fn new(pts: Pts) -> Self {
        FramesBatch {
            frames: HashMap::new(),
            pts,
        }
    }

    pub fn insert_frame(&mut self, input_id: InputID, frame: Arc<Frame>) {
        self.frames.insert(input_id, frame);
    }
}

pub struct Queue {
    internal_queue: Arc<Mutex<InternalQueue>>,
    check_queue_channel: (Sender<()>, Receiver<()>),
    output_framerate: Framerate,
    time_buffer_duration: Duration,
}

impl Queue {
    #[allow(dead_code)]
    pub fn new(output_framerate: Framerate) -> Self {
        Queue {
            internal_queue: Arc::new(Mutex::new(InternalQueue::new(output_framerate))),
            check_queue_channel: unbounded(),
            output_framerate,
            time_buffer_duration: Duration::from_millis(100),
        }
    }

    #[allow(dead_code)]
    pub fn add_input(&self, input_id: InputID) {
        let mut internal_queue = self.internal_queue.lock().unwrap();
        internal_queue.add_input(input_id);
    }

    #[allow(dead_code)]
    pub fn remove_input(&self, input_id: InputID) {
        let mut internal_queue = self.internal_queue.lock().unwrap();
        // TODO: gracefully remove input - wait until last enqueued frame PTS is smaller than output PTS
        internal_queue.remove_input(input_id);
    }

    #[allow(dead_code)]
    pub fn start(&self, sender: Sender<FramesBatch>) {
        // Starting timer
        let frame_interval_duration = self.output_framerate.get_interval_duration();
        let check_queue_sender = self.check_queue_channel.0.clone();
        let time_buffer_duration = self.time_buffer_duration;

        thread::spawn(move || {
            sleep(time_buffer_duration);
            let ticker = tick(frame_interval_duration);
            loop {
                ticker.recv().unwrap();
                check_queue_sender.send(()).unwrap();
            }
        });

        // Checking queue
        let start = Instant::now();
        let check_queue_receiver = self.check_queue_channel.1.clone();
        let internal_queue = self.internal_queue.clone();
        let interval_duration = self.output_framerate.get_interval_duration();
        let time_buffer_duration = self.time_buffer_duration;

        thread::spawn(move || loop {
            check_queue_receiver.recv().unwrap();

            let mut internal_queue = internal_queue.lock().unwrap();
            let buffer_pts = internal_queue.get_next_output_buffer_pts();
            let next_buffer_time =
                interval_duration * internal_queue.send_batches_counter + time_buffer_duration;

            if start.elapsed() > next_buffer_time
                || internal_queue.check_all_inputs_ready(buffer_pts)
            {
                let frames_batch = internal_queue.get_frames_batch(buffer_pts);
                sender.send(frames_batch).unwrap();

                internal_queue.drop_useless_frames();
            }
        });
    }

    pub fn enqueue_frame(&self, input_id: InputID, frame: Frame) -> Result<(), QueueError> {
        let mut internal_queue = self.internal_queue.lock().unwrap();

        internal_queue.enqueue_frame(input_id, frame)?;
        internal_queue.drop_pad_useless_frames(input_id)?;

        self.check_queue_channel.0.send(()).unwrap();

        Ok(())
    }
}
