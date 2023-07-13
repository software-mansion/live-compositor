mod internal_queue;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use crossbeam::channel::{tick, unbounded, Receiver, Sender};

use self::internal_queue::{InternalQueue, QueueError};

pub type InputID = u32;

/// nanoseconds
type Pts = u64;

/// TODO: This should be a rational.
#[derive(Debug, Clone, Copy)]
pub struct Framerate(pub u32);

impl Framerate {
    pub fn get_interval_duration(self) -> Duration {
        Duration::from_nanos((1_000_000_000 / self.0).into())
    }
}

#[allow(dead_code)]
pub struct MockFrame {
    y_plane: bytes::Bytes,
    u_plane: bytes::Bytes,
    v_plane: bytes::Bytes,
    pts: Pts,
}

#[allow(dead_code)]
pub struct FramesBatch {
    frames: HashMap<InputID, Arc<MockFrame>>,
    pts: Pts,
}

impl FramesBatch {
    pub fn new(pts: Pts) -> Self {
        FramesBatch {
            frames: HashMap::new(),
            pts,
        }
    }

    pub fn insert_frame(&mut self, input_id: InputID, frame: Arc<MockFrame>) {
        self.frames.insert(input_id, frame);
    }
}

pub struct Queue {
    internal_queue: Arc<Mutex<InternalQueue>>,
    check_queue_sender: Sender<()>,
    check_queue_receiver: Receiver<()>,
    output_framerate: Framerate,
    frames_batches_sent: u32,
    time_buffer_duration: Duration,
}

impl Queue {
    #[allow(dead_code)]
    pub fn new(output_framerate: Framerate) -> Self {
        let (check_queue_sender, check_queue_receiver) = unbounded();
        Queue {
            internal_queue: Arc::new(Mutex::new(InternalQueue::new())),
            check_queue_sender,
            check_queue_receiver,
            output_framerate,
            frames_batches_sent: 0,
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
    pub fn start(mut self, sender: Sender<FramesBatch>) {
        // Starting timer
        let frame_interval_duration = self.output_framerate.get_interval_duration();
        let ticker = tick(frame_interval_duration);

        let check_queue_sender = self.check_queue_sender.clone();
        thread::spawn(move || loop {
            ticker.recv().unwrap();
            check_queue_sender.send(()).unwrap();
        });

        let start = Instant::now();

        let check_queue_receiver = self.check_queue_receiver.clone();
        // Checking queue
        thread::spawn(move || loop {
            check_queue_receiver.recv().unwrap();

            let mut internal_queue = self.internal_queue.lock().unwrap();
            let buffer_pts = self.get_next_output_buffer_pts();
            let next_buffer_time = self.output_framerate.get_interval_duration()
                * self.frames_batches_sent
                + self.time_buffer_duration;

            if start.elapsed() > next_buffer_time
                || internal_queue.check_all_inputs_ready(buffer_pts)
            {
                let frames_batch = internal_queue.get_frames_batch(buffer_pts);
                sender.send(frames_batch).unwrap();
                self.frames_batches_sent += 1;

                internal_queue.drop_useless_frames(self.get_next_output_buffer_pts());
            }
        });
    }

    #[allow(dead_code)]
    pub fn enqueue_frame(&self, input_id: InputID, frame: MockFrame) -> Result<(), QueueError> {
        let mut internal_queue = self.internal_queue.lock().unwrap();

        internal_queue.enqueue_frame(input_id, frame)?;
        internal_queue.drop_pad_useless_frames(input_id, self.get_next_output_buffer_pts())?;

        self.check_queue_sender.send(()).unwrap();

        Ok(())
    }

    fn get_next_output_buffer_pts(&self) -> Pts {
        let nanoseconds_in_second = 1_000_000_000;
        (nanoseconds_in_second * (self.frames_batches_sent as u64 + 1))
            / self.output_framerate.0 as u64
    }
}
