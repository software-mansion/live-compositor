use std::{
    sync::Arc,
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use compositor_render::AudioSamplesSet;
use crossbeam_channel::{tick, Sender};
use log::warn;

use super::Queue;

pub struct Options {
    pub buffer_duration: Duration,
    pub pushed_chunk_length: Duration,
    pub clock_start: Instant,
}

pub struct AudioQueueThread {
    queue: Arc<Queue>,
    sender: Sender<AudioSamplesSet>,
    buffer_duration: Duration,
    chunk_duration: Duration,
    chunks_counter: u32,
    clock_start: Instant,
}

impl AudioQueueThread {
    pub fn new(queue: Arc<Queue>, sender: Sender<AudioSamplesSet>, opts: Options) -> Self {
        AudioQueueThread {
            queue,
            sender,
            buffer_duration: opts.buffer_duration,
            chunk_duration: opts.pushed_chunk_length,
            clock_start: opts.clock_start,
            chunks_counter: 0,
        }
    }

    pub fn spawn(mut self) -> JoinHandle<()> {
        thread::spawn(move || self.run())
    }

    fn run(&mut self) {
        thread::sleep(self.buffer_duration);
        let ticker = tick(self.chunk_duration);
        loop {
            ticker.recv().unwrap();
            let pts = self.chunk_duration * self.chunks_counter;
            let samples = self
                .queue
                .audio_queue
                .lock()
                .unwrap()
                .pop_samples_set((pts, pts + self.chunk_duration), self.clock_start);
            if self.sender.try_send(samples).is_err() {
                warn!("Queue failed to push audio chunks. Channel is blocked.")
            }
            self.chunks_counter += 1;
        }
    }
}
