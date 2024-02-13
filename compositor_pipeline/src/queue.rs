mod audio_queue;
mod audio_queue_thread;
mod utils;
mod video_queue;
mod video_queue_thread;

use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use compositor_render::{AudioSamplesSet, FrameSet, Framerate, InputId};
use crossbeam_channel::{unbounded, Receiver, Sender};

use crate::pipeline::decoder::DecodedDataReceiver;

use self::{
    audio_queue::AudioQueue, audio_queue_thread::AudioQueueThread, video_queue::VideoQueue,
    video_queue_thread::VideoQueueThread,
};

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

    pub fn add_input(&self, input_id: &InputId, receiver: DecodedDataReceiver) {
        if let Some(receiver) = receiver.video {
            self.video_queue
                .lock()
                .unwrap()
                .add_input(input_id, receiver);
        };
        if let Some(receiver) = receiver.audio {
            self.audio_queue
                .lock()
                .unwrap()
                .add_input(input_id, receiver);
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

    pub fn subscribe_input_listener(&self, input_id: &InputId, callback: Box<dyn FnOnce() + Send>) {
        self.video_queue
            .lock()
            .unwrap()
            .subscribe_input_listener(input_id, callback)
    }
}
