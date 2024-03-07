use std::{
    collections::BinaryHeap,
    ops::Add,
    sync::{Arc, MutexGuard},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use crossbeam_channel::{select, tick, Receiver, Sender};
use tracing::{debug, info, trace, warn};

use super::{
    audio_queue::AudioQueue, video_queue::VideoQueue, Queue, QueueAudioOutput, QueueVideoOutput,
    ScheduledEvent,
};

pub(super) struct QueueThread {
    queue: Arc<Queue>,
    start_receiver: Receiver<QueueStartEvent>,
    scheduled_event_receiver: Receiver<ScheduledEvent>,
    scheduled_events: BinaryHeap<ScheduledEvent>,
}

pub(super) struct QueueStartEvent {
    pub(super) video_sender: Sender<QueueVideoOutput>,
    pub(super) audio_sender: Sender<QueueAudioOutput>,
    pub(super) start_time: Instant,
}

impl QueueThread {
    pub fn new(
        queue: Arc<Queue>,
        start_receiver: Receiver<QueueStartEvent>,
        scheduled_event_receiver: Receiver<ScheduledEvent>,
    ) -> Self {
        Self {
            queue,
            start_receiver,
            scheduled_event_receiver,
            scheduled_events: BinaryHeap::new(),
        }
    }

    pub fn spawn(self) -> JoinHandle<()> {
        thread::Builder::new()
            .name("Queue thread".to_string())
            .spawn(move || self.run())
            .unwrap()
    }

    fn run(mut self) {
        let ticker = tick(self.queue.output_framerate.get_interval_duration());
        loop {
            select! {
                recv(ticker) -> _ => {
                    self.cleanup_old_data()
                },
                recv(self.scheduled_event_receiver) -> event => {
                    self.scheduled_events.push(event.unwrap())
                }
                recv(self.start_receiver) -> start_event => {
                    QueueThreadAfterStart::new(self, start_event.unwrap()).run();
                    return;
                },
            };
        }
    }

    fn cleanup_old_data(&mut self) {
        // Drop old frames as if start was happening now.
        self.queue
            .video_queue
            .lock()
            .unwrap()
            .drop_old_frames(Duration::ZERO, Instant::now());
        self.queue
            .audio_queue
            .lock()
            .unwrap()
            .drop_old_samples(Duration::ZERO, Instant::now())
    }
}

struct QueueThreadAfterStart {
    audio_processor: AudioQueueProcessor,
    video_processor: VideoQueueProcessor,
    scheduled_event_receiver: Receiver<ScheduledEvent>,
    scheduled_events: BinaryHeap<ScheduledEvent>,
}

impl QueueThreadAfterStart {
    fn new(queue_thread: QueueThread, start_event: QueueStartEvent) -> Self {
        Self {
            audio_processor: AudioQueueProcessor {
                queue: queue_thread.queue.clone(),
                sender: start_event.audio_sender,
                chunks_counter: 0,
                queue_start_time: start_event.start_time,
            },
            video_processor: VideoQueueProcessor {
                queue: queue_thread.queue,
                sender: start_event.video_sender,
                sent_batches_counter: 0,
                queue_start_time: start_event.start_time,
            },
            scheduled_event_receiver: queue_thread.scheduled_event_receiver,
            scheduled_events: queue_thread.scheduled_events,
        }
    }

    fn run(mut self) {
        let ticker = tick(Duration::from_millis(10));

        loop {
            select! {
                recv(ticker) -> _ => (),
                recv(self.scheduled_event_receiver) -> event => {
                    self.scheduled_events.push(event.unwrap());
                    continue;
                }
            };
            loop {
                let audio_pts_range = self.audio_processor.next_buffer_pts_range();
                let video_pts = self.video_processor.next_buffer_pts();
                let event_pts = self.scheduled_events.peek().map(|e| e.pts);

                if let Some(true) = event_pts.map(|event_pts: Duration| {
                    event_pts < video_pts && event_pts < audio_pts_range.0
                }) {
                    info!("Handle scheduled event for PTS={:?}", event_pts);
                    if let Some(ScheduledEvent { callback, .. }) = self.scheduled_events.pop() {
                        callback()
                    }
                } else if video_pts > audio_pts_range.0 {
                    trace!(pts_range=?audio_pts_range, "Try to push audio samples for.");
                    if self
                        .audio_processor
                        .try_push_next_sample_batch(audio_pts_range)
                        .is_none()
                    {
                        break;
                    }
                } else {
                    trace!(pts=?video_pts, "Try to push video frames.");
                    if self
                        .video_processor
                        .try_push_next_frame_set(video_pts)
                        .is_none()
                    {
                        break;
                    }
                }
            }
        }
    }
}

struct VideoQueueProcessor {
    queue: Arc<Queue>,
    sent_batches_counter: u32,
    queue_start_time: Instant,
    sender: Sender<QueueVideoOutput>,
}

impl VideoQueueProcessor {
    fn next_buffer_pts(&self) -> Duration {
        Duration::from_secs_f64(
            self.sent_batches_counter as f64 * self.queue.output_framerate.den as f64
                / self.queue.output_framerate.num as f64,
        )
    }

    fn should_push_for_pts(&self, pts: Duration, queue: &mut MutexGuard<VideoQueue>) -> bool {
        if !self.queue.ahead_of_time_processing && self.queue_start_time.add(pts) > Instant::now() {
            return false;
        }
        if queue.check_all_inputs_ready_for_pts(pts, self.queue_start_time) {
            return true;
        }
        if !queue.check_all_required_inputs_ready_for_pts(pts, self.queue_start_time) {
            return false;
        }
        self.queue_start_time.add(pts) < Instant::now()
    }

    fn send_output_frames(&mut self, frames_batch: QueueVideoOutput, is_required: bool) {
        let pts = frames_batch.pts;
        debug!(?pts, "Pushing video frames.");
        if is_required {
            self.sender.send(frames_batch).unwrap()
        } else {
            let send_deadline = self.queue_start_time.add(frames_batch.pts);
            if self
                .sender
                .send_deadline(frames_batch, send_deadline)
                .is_err()
            {
                warn!(?pts, "Dropping video frame on queue output.");
            }
        }
        self.sent_batches_counter += 1
    }

    /// Some(()) - Successfully pushed new frame (or dropped it).
    /// None - Nothing to push.
    fn try_push_next_frame_set(&mut self, next_buffer_pts: Duration) -> Option<()> {
        let mut internal_queue = self.queue.video_queue.lock().unwrap();

        let should_push_next_frame = self.should_push_for_pts(next_buffer_pts, &mut internal_queue);
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
}

struct AudioQueueProcessor {
    queue: Arc<Queue>,
    chunks_counter: u32,
    queue_start_time: Instant,
    sender: Sender<QueueAudioOutput>,
}

impl AudioQueueProcessor {
    fn next_buffer_pts_range(&self) -> (Duration, Duration) {
        (
            self.queue.audio_chunk_duration * self.chunks_counter,
            self.queue.audio_chunk_duration * (self.chunks_counter + 1),
        )
    }

    fn should_push_for_pts_range(
        &self,
        pts_range: (Duration, Duration),
        queue: &mut MutexGuard<AudioQueue>,
    ) -> bool {
        if !self.queue.ahead_of_time_processing
            && self.queue_start_time.add(pts_range.0) > Instant::now()
        {
            return false;
        }
        if queue.check_all_inputs_ready_for_pts(pts_range, self.queue_start_time) {
            return true;
        }
        if !queue.check_all_required_inputs_ready_for_pts(pts_range, self.queue_start_time) {
            return false;
        }
        self.queue_start_time.add(pts_range.0) < Instant::now()
    }

    /// Some(()) - Successfully pushed new batch (or dropped it).
    /// None - Nothing to push.
    fn try_push_next_sample_batch(
        &mut self,
        next_buffer_pts_range: (Duration, Duration),
    ) -> Option<()> {
        let mut internal_queue = self.queue.audio_queue.lock().unwrap();

        let should_push_next_batch =
            self.should_push_for_pts_range(next_buffer_pts_range, &mut internal_queue);
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

    fn send_output_batch(&mut self, samples: QueueAudioOutput, is_required: bool) {
        let pts_range = (samples.start_pts, samples.end_pts);
        debug!(?pts_range, "Pushing audio samples.");
        if is_required {
            self.sender.send(samples).unwrap()
        } else if self.sender.try_send(samples).is_err() {
            warn!(?pts_range, "Dropping audio batch on queue output.")
        }
        self.chunks_counter += 1;
    }
}
