use std::{
    ops::Add,
    sync::{Arc, MutexGuard},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use compositor_render::{FrameSet, InputId};
use crossbeam_channel::{select, tick, Receiver, Sender};
use log::warn;

use super::{video_queue::VideoQueue, Queue};

pub(super) struct VideoQueueThread {
    queue: Arc<Queue>,
    start_receiver: Receiver<VideoQueueStartEvent>,
}

pub(super) struct VideoQueueStartEvent {
    pub(super) sender: Sender<FrameSet<InputId>>,
    pub(super) start_time: Instant,
}

struct VideoQueueThreadAfterStart {
    queue: Arc<Queue>,
    sender: Sender<FrameSet<InputId>>,
    sent_batches_counter: u32,
    queue_start_time: Instant,
}

impl VideoQueueThread {
    pub fn new(queue: Arc<Queue>, start_receiver: Receiver<VideoQueueStartEvent>) -> Self {
        Self {
            queue,
            start_receiver,
        }
    }

    pub fn spawn(self) -> JoinHandle<()> {
        thread::Builder::new()
            .name("Video queue thread".to_string())
            .spawn(move || self.run())
            .unwrap()
    }

    fn run(mut self) {
        let ticker = tick(self.queue.output_framerate.get_interval_duration());
        loop {
            select! {
                recv(ticker) -> _ => {
                    self.cleanup_old_frames()
                },
                recv(self.start_receiver) -> start_event => {
                    VideoQueueThreadAfterStart::new(self, start_event.unwrap()).run();
                    return;
                },
            };
        }
    }

    fn cleanup_old_frames(&mut self) {
        // Drop old frames as if start was happening now.
        self.queue
            .video_queue
            .lock()
            .unwrap()
            .drop_old_frames(Duration::ZERO, Instant::now())
    }
}

impl VideoQueueThreadAfterStart {
    fn new(queue_thread: VideoQueueThread, start_event: VideoQueueStartEvent) -> Self {
        Self {
            queue: queue_thread.queue,
            sender: start_event.sender,
            sent_batches_counter: 0,
            queue_start_time: start_event.start_time,
        }
    }

    fn run(mut self) {
        let ticker = tick(self.queue.output_framerate.get_interval_duration());

        loop {
            ticker.recv().unwrap();
            while self.try_push_next_frame().is_some() {}
        }
    }

    fn should_push_pts(&self, pts: Duration, queue: &mut MutexGuard<VideoQueue>) -> bool {
        if !self.queue.ahead_of_time_processing && self.queue_start_time.add(pts) > Instant::now() {
            return false;
        }
        if queue.check_all_inputs_ready_for_pts(pts, self.queue_start_time) {
            return true;
        }
        if queue.has_required_inputs_for_pts(pts, self.queue_start_time) {
            return false;
        }
        self.queue_start_time.add(pts) < Instant::now()
    }

    fn send_output_frames(&mut self, frames_batch: FrameSet<InputId>, is_required: bool) {
        if is_required {
            self.sender.send(frames_batch).unwrap()
        } else {
            let send_deadline = self.queue_start_time.add(frames_batch.pts);
            if self
                .sender
                .send_deadline(frames_batch, send_deadline)
                .is_err()
            {
                warn!("Dropping video frame on queue output.");
            }
        }
        self.sent_batches_counter += 1
    }

    /// Some(()) - Successfully pushed new frame (or dropped it).
    /// None - Nothing to push.
    fn try_push_next_frame(&mut self) -> Option<()> {
        let mut internal_queue = self.queue.video_queue.lock().unwrap();
        let next_buffer_pts = self.get_next_output_buffer_pts();

        let should_push_next_frame = self.should_push_pts(next_buffer_pts, &mut internal_queue);
        if !should_push_next_frame {
            return None;
        }

        let frames_batch = internal_queue.get_frames_batch(next_buffer_pts, self.queue_start_time);
        for input_id in frames_batch.frames.keys() {
            internal_queue.call_input_listeners(input_id)
        }

        let is_required =
            internal_queue.has_required_inputs_for_pts(next_buffer_pts, self.queue_start_time);
        drop(internal_queue);

        // potentially infinitely blocking if output is not consumed
        // and one of the stream is "required"
        self.send_output_frames(frames_batch, is_required);

        Some(())
    }

    fn get_next_output_buffer_pts(&self) -> Duration {
        Duration::from_secs_f64(
            self.sent_batches_counter as f64 * self.queue.output_framerate.den as f64
                / self.queue.output_framerate.num as f64,
        )
    }
}
