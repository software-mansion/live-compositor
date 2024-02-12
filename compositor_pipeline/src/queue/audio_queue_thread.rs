use std::{
    ops::Add,
    sync::{Arc, MutexGuard},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use compositor_render::AudioSamplesSet;
use crossbeam_channel::{select, tick, Receiver, Sender};
use log::warn;

use super::{audio_queue::AudioQueue, Queue, UnlimitedDuration};

pub(super) struct AudioQueueStartEvent {
    pub(super) sender: Sender<AudioSamplesSet>,
    pub(super) start_time: Instant,
}

pub struct AudioQueueThread {
    queue: Arc<Queue>,
    start_receiver: Receiver<AudioQueueStartEvent>,
}

struct AudioQueueThreadAfterStart {
    queue: Arc<Queue>,
    sender: Sender<AudioSamplesSet>,
    chunks_counter: u32,
    queue_start_time: Instant,
}

impl AudioQueueThread {
    pub fn new(queue: Arc<Queue>, start_receiver: Receiver<AudioQueueStartEvent>) -> Self {
        AudioQueueThread {
            queue,
            start_receiver,
        }
    }

    pub fn spawn(self) -> JoinHandle<()> {
        thread::Builder::new()
            .name("Audio queue thread".to_string())
            .spawn(move || self.run())
            .unwrap()
    }

    fn run(mut self) -> ! {
        let ticker = tick(self.queue.buffer_duration);
        loop {
            select! {
                recv(ticker) -> _ => {
                    self.cleanup_old_samples()
                },
                recv(self.start_receiver) -> start_event => {
                    AudioQueueThreadAfterStart::new(self, start_event.unwrap()).run();
                },
            };
        }
    }

    fn cleanup_old_samples(&mut self) {
        // Drop old frames as if start was happening now.
        self.queue
            .video_queue
            .lock()
            .unwrap()
            .drop_old_frames(Duration::ZERO, Instant::now())
    }
}

impl AudioQueueThreadAfterStart {
    fn new(queue_thread: AudioQueueThread, start_event: AudioQueueStartEvent) -> Self {
        Self {
            queue: queue_thread.queue,
            sender: start_event.sender,
            chunks_counter: 0,
            queue_start_time: start_event.start_time,
        }
    }

    fn run(&mut self) -> ! {
        let ticker = tick(self.queue.audio_chunk_duration);
        loop {
            ticker.recv().unwrap();
            while self.try_push_next_sample_batch().is_some() {}
        }
    }

    fn should_push_pts(
        &self,
        pts_range: (Duration, Duration),
        queue: &mut MutexGuard<AudioQueue>,
    ) -> bool {
        if let UnlimitedDuration::Finite(duration) = self.queue.ahead_of_time_processing_buffer {
            if self.queue_start_time.add(pts_range.0) > Instant::now() + duration {
                return false;
            }
        }
        if queue.check_all_inputs_ready_for_pts(pts_range, self.queue_start_time) {
            return true;
        }
        if queue.has_required_inputs_for_pts(pts_range.0, self.queue_start_time) {
            return false;
        }
        self.queue_start_time.add(pts_range.0) < Instant::now()
    }

    /// Some(()) - Successfully pushed new batch (or dropped it).
    /// None - Nothing to push.
    fn try_push_next_sample_batch(&mut self) -> Option<()> {
        let next_buffer_pts_range = (
            self.queue.audio_chunk_duration * self.chunks_counter,
            self.queue.audio_chunk_duration * (self.chunks_counter + 1),
        );

        let mut internal_queue = self.queue.audio_queue.lock().unwrap();

        let should_push_next_batch =
            self.should_push_pts(next_buffer_pts_range, &mut internal_queue);
        if !should_push_next_batch {
            return None;
        }

        let samples = internal_queue.pop_samples_set(next_buffer_pts_range, self.queue_start_time);
        let is_required = internal_queue
            .has_required_inputs_for_pts(next_buffer_pts_range.0, self.queue_start_time);
        drop(internal_queue);

        self.send_output_batch(samples, is_required);

        Some(())
    }

    fn send_output_batch(&mut self, samples: AudioSamplesSet, is_required: bool) {
        if is_required {
            self.sender.send(samples).unwrap()
        } else if self.sender.try_send(samples).is_err() {
            warn!("Dropping audio batch on queue output.")
        }
        self.chunks_counter += 1;
    }
}
