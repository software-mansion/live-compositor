mod audio_queue;
mod queue_thread;
mod utils;
mod video_queue;

use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use compositor_render::{AudioSamplesSet, FrameSet, Framerate, InputId};
use crossbeam_channel::{bounded, Sender};

use crate::pipeline::decoder::DecodedDataReceiver;

use self::{
    audio_queue::AudioQueue,
    queue_thread::{QueueStartEvent, QueueThread},
    video_queue::VideoQueue,
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

    /// Define if queue should process frames if all inputs are ready.
    ahead_of_time_processing: bool,

    start_sender: Mutex<Option<Sender<QueueStartEvent>>>,
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
        let (queue_start_sender, queue_start_receiver) = bounded(0);
        let buffer_duration = DEFAULT_BUFFER_DURATION;
        let queue = Arc::new(Queue {
            video_queue: Mutex::new(VideoQueue::new(buffer_duration)),
            output_framerate: opts.output_framerate,
            ahead_of_time_processing: opts.ahead_of_time_processing,
            audio_queue: Mutex::new(AudioQueue::new(buffer_duration)),
            audio_chunk_duration: DEFAULT_AUDIO_CHUNK_DURATION,
            start_sender: Mutex::new(Some(queue_start_sender)),
        });

        QueueThread::new(queue.clone(), queue_start_receiver).spawn();

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
        if let Some(sender) = self.start_sender.lock().unwrap().take() {
            sender
                .send(QueueStartEvent {
                    audio_sender,
                    video_sender,
                    start_time: Instant::now(),
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
