use std::{
    ops::Add,
    sync::Arc,
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use compositor_common::{scene::InputId, Framerate};
use compositor_render::frame_set::FrameSet;
use crossbeam_channel::{tick, Sender};

use super::Queue;

pub struct Options {
    pub buffer_duration: Duration,
    pub tick_duration: Duration,
    pub clock_start: Instant,
    pub output_framerate: Framerate,
}

pub struct QueueThread {
    queue: Arc<Queue>,
    sender: Sender<FrameSet<InputId>>,
    opts: Options,
    sent_batches_counter: u32,
    output_frame_offset: Duration,
}

impl QueueThread {
    pub fn new(queue: Arc<Queue>, sender: Sender<FrameSet<InputId>>, opts: Options) -> Self {
        let output_frame_offset = opts.clock_start.elapsed();
        Self {
            queue,
            sender,
            opts,
            output_frame_offset,
            sent_batches_counter: 0,
        }
    }

    pub fn spawn(mut self) -> JoinHandle<()> {
        thread::spawn(move || self.run())
    }

    fn should_push_pts(&self, pts: Duration) -> bool {
        self.opts.clock_start.add(pts) < Instant::now()
    }

    fn run(&mut self) {
        // This is just in case QueueThread is not spawned after the creation
        self.output_frame_offset = self.opts.clock_start.elapsed();
        self.start_ticker();

        loop {
            self.queue.check_queue_channel.1.recv().unwrap();
            self.on_queue_event()
        }
    }

    fn on_queue_event(&mut self) {
        let mut internal_queue = self.queue.internal_queue.lock().unwrap();
        let next_buffer_pts = self.get_next_output_buffer_pts();

        let ready_to_push = internal_queue.check_all_inputs_ready(next_buffer_pts)
            || self.should_push_pts(next_buffer_pts);
        if !ready_to_push {
            return;
        }

        let frames_batch = internal_queue.get_frames_batch(next_buffer_pts);
        for input_id in frames_batch.frames.keys() {
            internal_queue.call_input_listeners(input_id)
        }
        self.sender.send(frames_batch).unwrap();
        self.sent_batches_counter += 1;
    }

    fn get_next_output_buffer_pts(&self) -> Duration {
        Duration::from_secs_f64(
            self.sent_batches_counter as f64 / self.opts.output_framerate.0 as f64,
        ) + self.output_frame_offset
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
