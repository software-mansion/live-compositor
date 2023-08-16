use std::{
    sync::{Arc, MutexGuard},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use compositor_common::scene::InputId;
use compositor_render::frame_set::FrameSet;
use crossbeam_channel::{tick, Sender};

use super::{internal_queue::InternalQueue, Queue};

pub struct Options {
    pub buffer_duration: Duration,
    pub tick_duration: Duration,
}

pub struct QueueThread {
    queue: Arc<Queue>,
    sender: Sender<FrameSet<InputId>>,
    opts: Options,
}

impl QueueThread {
    pub fn new(queue: Arc<Queue>, sender: Sender<FrameSet<InputId>>, opts: Options) -> Self {
        Self {
            queue,
            sender,
            opts,
        }
    }

    pub fn spawn(mut self) -> JoinHandle<()> {
        thread::spawn(move || self.run())
    }

    fn run(&mut self) {
        // Wait for first frame
        self.queue.check_queue_channel.1.recv().unwrap();
        thread::sleep(self.opts.buffer_duration);

        self.start_ticker();

        let start = Instant::now();
        loop {
            self.queue.check_queue_channel.1.recv().unwrap();

            let mut internal_queue = self.queue.internal_queue.lock().unwrap();
            let buffer_pts = internal_queue.get_next_output_buffer_pts();

            if start.elapsed() + self.opts.buffer_duration > buffer_pts
                || internal_queue.check_all_inputs_ready(buffer_pts)
            {
                let frames_batch = internal_queue.get_frames_batch(buffer_pts);
                self.send_frames(frames_batch, &mut internal_queue);
                internal_queue.drop_useless_frames();
            }
        }
    }

    fn send_frames(
        &self,
        frames_batch: FrameSet<InputId>,
        internal_queue: &mut MutexGuard<InternalQueue>,
    ) {
        for input_id in frames_batch.frames.keys() {
            internal_queue.call_input_listeners(input_id)
        }
        self.sender.send(frames_batch).unwrap();
    }

    fn start_ticker(&self) {
        let check_queue_sender = self.queue.check_queue_channel.0.clone();
        let tick_duration = self.opts.tick_duration;
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
