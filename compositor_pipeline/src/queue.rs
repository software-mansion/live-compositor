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
use crossbeam_channel::{bounded, Sender};

use crate::pipeline::decoder::DecodedDataReceiver;

use self::{
    audio_queue::AudioQueue,
    audio_queue_thread::{AudioQueueStartEvent, AudioQueueThread},
    video_queue::VideoQueue,
    video_queue_thread::{VideoQueueStartEvent, VideoQueueThread},
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
    audio_queue: Mutex<AudioQueue>,

    output_framerate: Framerate,

    /// Duration of queue output samples set.
    audio_chunk_duration: Duration,

    /// - When new input is connected and sends the first frame we want to wait
    /// buffer_duration before sending first frame of that input.
    /// - When pipeline is started we want to start with a frame that was receive
    /// `buffer_duration` time ago.
    buffer_duration: Duration,
    /// Define if queue should process frames if all inputs are ready.
    ahead_of_time_processing: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct InputOptions {
    pub required: bool,
    /// Relative offset this input stream should have to the clock that
    /// starts when pipeline is started.
    pub offset: Option<Duration>,
}

#[derive(Debug, Clone, Copy)]
pub struct QueueOptions {
    pub ahead_of_time_processing: bool,
    pub output_framerate: Framerate,
}

impl Queue {
    pub fn new(opts: QueueOptions) -> Arc<Self> {
        let (video_queue_start_sender, video_queue_start_receiver) = bounded(0);
        let (audio_queue_start_sender, audio_queue_start_receiver) = bounded(0);
        let queue = Arc::new(Queue {
            video_queue: Mutex::new(VideoQueue::new(video_queue_start_sender)),
            output_framerate: opts.output_framerate,
            ahead_of_time_processing: opts.ahead_of_time_processing,
            buffer_duration: DEFAULT_BUFFER_DURATION,
            audio_queue: Mutex::new(AudioQueue::new(audio_queue_start_sender)),
            audio_chunk_duration: DEFAULT_AUDIO_CHUNK_DURATION,
        });

        VideoQueueThread::new(queue.clone(), video_queue_start_receiver).spawn();
        AudioQueueThread::new(queue.clone(), audio_queue_start_receiver).spawn();

        queue
    }

    pub fn add_input(&self, input_id: &InputId, receiver: DecodedDataReceiver, opts: InputOptions) {
        if let Some(receiver) = receiver.video {
            self.video_queue
                .lock()
                .unwrap()
                .add_input(input_id, receiver, opts);
        };
        if let Some(receiver) = receiver.audio {
            self.audio_queue
                .lock()
                .unwrap()
                .add_input(input_id, receiver, opts);
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
        let start_time = Instant::now();
        let sender = self.video_queue.lock().unwrap().take_start_sender();
        if let Some(sender) = sender {
            sender
                .send(VideoQueueStartEvent {
                    sender: video_sender,
                    start_time,
                })
                .unwrap()
        }

        let sender = self.audio_queue.lock().unwrap().take_start_sender();
        if let Some(sender) = sender {
            sender
                .send(AudioQueueStartEvent {
                    sender: audio_sender,
                    start_time,
                })
                .unwrap()
        }
    }

    pub fn subscribe_input_listener(&self, input_id: &InputId, callback: Box<dyn FnOnce() + Send>) {
        self.video_queue
            .lock()
            .unwrap()
            .subscribe_input_listener(input_id, callback)
    }
}
