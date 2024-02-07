mod audio_queue;
mod audio_queue_thread;
mod video_queue;
mod video_queue_thread;

use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use compositor_render::{
    error::ErrorStack, AudioSamples, AudioSamplesBatch, AudioSamplesSet, Frame, FrameSet,
    Framerate, InputId,
};
use crossbeam_channel::{unbounded, Receiver, Sender};
use log::error;
use thiserror::Error;

use crate::pipeline::AudioChannels;

use self::{
    audio_queue::AudioQueue, audio_queue_thread::AudioQueueThread, video_queue::VideoQueue,
    video_queue_thread::VideoQueueThread,
};

pub struct InputType {
    pub input_id: InputId,
    pub video: Option<()>,
    pub audio: Option<AudioOptions>,
}

pub struct AudioOptions {
    pub sample_rate: u32,
    pub channels: AudioChannels,
}

#[derive(Error, Debug)]
pub enum QueueError {
    #[error("the input id `{:#?}` is unknown", 0)]
    UnknownInputId(InputId),
    #[error(
        "expected samples in {:#?} channel format, but received samples {:#?}",
        expected,
        received
    )]
    MismatchedSamplesChannels {
        expected: AudioChannels,
        received: AudioSamples,
    },
    #[error(
        "expected samples with {} sample rate, but received samples of rate {}",
        expected,
        received
    )]
    MismatchedSampleRate { expected: u32, received: u32 },
}

const DEFAULT_BUFFER_DURATION: Duration = Duration::from_millis(16 * 5); // about 5 frames at 60 fps
const DEFAULT_AUDIO_CHUNK_DURATION: Duration = Duration::from_millis(20); // typical audio packet size

/// Queue is responsible for consuming frames from different inputs and producing
/// sets of frames from all inputs in a single batch.
///
/// PTS after frame is enqueued:
/// - PTS=0 should represent clock_start instant for each input.
/// - Queue is force pushing frames when time that represents the next PTS
///   is earlier than Instant.now(). We don't need any buffering because we
///   are blocking on the first frame of each input.
/// - Technically real_pts = frame.pts - buffer_duration, but because relative value
///   does not matter we don't need to take that into account.
pub struct Queue {
    video_queue: Mutex<VideoQueue>,
    check_video_queue_channel: (Sender<()>, Receiver<()>),
    output_framerate: Framerate,

    audio_queue: Mutex<AudioQueue>,
    /// Duration of queue output samples set.
    audio_chunk_duration: Duration,
    /// - When new input is connected and sends the first frame we want to wait
    /// buffer_duration before sending first frame of that input.
    /// - When pipeline is started we want to start with a frame that was receive
    /// `buffer_duration` time ago
    buffer_duration: Duration,

    /// Base time that is used to synchronize PTS value of received frame to
    /// the same clock. When enqueueing the frame we are modifying it's PTS
    /// using (per input) offset calculated based on this clock.
    ///
    /// The end goal is that resulting PTS should be in a time frame where PTS
    /// is equivalent to clock_start time of real time.
    clock_start: Instant,
}

impl Queue {
    pub fn new(output_framerate: Framerate) -> Self {
        Queue {
            video_queue: Mutex::new(VideoQueue::new()),
            check_video_queue_channel: unbounded(),
            output_framerate,
            buffer_duration: DEFAULT_BUFFER_DURATION,
            clock_start: Instant::now(),
            audio_queue: Mutex::new(AudioQueue::new()),
            audio_chunk_duration: DEFAULT_AUDIO_CHUNK_DURATION,
        }
    }

    pub fn add_input(&self, options: InputType) {
        if options.video.is_some() {
            self.video_queue
                .lock()
                .unwrap()
                .add_input(options.input_id.clone());
        };
        if let Some(_audio_options) = options.audio {
            self.audio_queue.lock().unwrap().add_input(options.input_id);
        }
    }

    pub fn remove_input(&self, input_id: &InputId) {
        self.video_queue.lock().unwrap().remove_input(input_id);
        self.audio_queue.lock().unwrap().remove_input(input_id);
    }

    pub fn start(
        self: &Arc<Self>,
        video_sender: Sender<FrameSet<InputId>>,
        audio_sender: Sender<AudioSamplesSet>,
    ) {
        let queue = self.clone();
        let tick_duration = self.output_framerate.get_interval_duration();

        VideoQueueThread::new(
            queue.clone(),
            video_sender,
            video_queue_thread::Options {
                tick_duration,
                buffer_duration: self.buffer_duration,
                output_framerate: self.output_framerate,
                clock_start: self.clock_start,
            },
        )
        .spawn();

        AudioQueueThread::new(
            queue,
            audio_sender,
            audio_queue_thread::Options {
                buffer_duration: self.buffer_duration,
                pushed_chunk_length: self.audio_chunk_duration,
                clock_start: self.clock_start,
            },
        )
        .spawn();
    }

    pub fn enqueue_audio_samples(
        &self,
        input_id: InputId,
        samples: AudioSamplesBatch,
    ) -> Result<(), QueueError> {
        let is_first_batch_for_input = !self
            .audio_queue
            .lock()
            .unwrap()
            .did_receive_samples(&input_id);
        if is_first_batch_for_input {
            thread::sleep(self.buffer_duration)
        }
        self.audio_queue
            .lock()
            .unwrap()
            .enqueue_samples(input_id, samples, self.clock_start)
    }

    pub fn enqueue_video_frame(&self, input_id: InputId, frame: Frame) -> Result<(), QueueError> {
        let is_first_frame_for_input = !self
            .video_queue
            .lock()
            .unwrap()
            .did_receive_frame(&input_id);
        if is_first_frame_for_input {
            // Sleep here ensures that we will buffer `self.buffer_duration` on each input.
            // It also makes calculation easier because PTS of frames will be already offset
            // by a correct value.
            thread::sleep(self.buffer_duration);
        }

        let mut internal_queue = self.video_queue.lock().unwrap();

        internal_queue.enqueue_frame(input_id.clone(), frame, self.clock_start)?;

        // We don't know when pipeline is started, so we can't resolve real_next_pts,
        // but we can remove frames based on estimated PTS. This only works if queue
        // is able to push frames in real time and is never behind more than one frame.
        let framerate_tick = Duration::from_secs_f64(
            self.output_framerate.den as f64 / self.output_framerate.num as f64,
        );
        let estimated_pts = self.clock_start.elapsed() - framerate_tick;
        if let Err(err) = internal_queue.drop_old_frames_by_input_id(&input_id, estimated_pts) {
            error!(
                "Failed to drop frames on input {}:\n{}",
                input_id,
                ErrorStack::new(&err).into_string()
            )
        }

        self.check_video_queue_channel.0.send(()).unwrap();

        Ok(())
    }

    pub fn subscribe_input_listener(&self, input_id: InputId, callback: Box<dyn FnOnce() + Send>) {
        self.video_queue
            .lock()
            .unwrap()
            .subscribe_input_listener(input_id, callback)
    }
}
