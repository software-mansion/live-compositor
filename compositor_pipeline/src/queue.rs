mod audio_queue;
mod queue_thread;
mod utils;
mod video_queue;

use std::{
    cmp::{self, Ordering},
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use compositor_render::{Frame, FrameSet, Framerate, InputId};
use crossbeam_channel::{bounded, Sender};

use crate::{
    audio_mixer::{InputSamples, InputSamplesSet},
    pipeline::decoder::DecodedDataReceiver,
};

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
/// - PTS of inputs streams can be in any frame of reference.
/// - PTS of frames stored in queue are in a frame of reference where PTS=0 represents.
///   first frame/packet.
/// - PTS of output frames is in a frame of reference where PTS=0 represents
///   start request.
pub struct Queue {
    video_queue: Mutex<VideoQueue>,
    audio_queue: Mutex<AudioQueue>,

    output_framerate: Framerate,

    /// Duration of queue output samples set.
    audio_chunk_duration: Duration,

    /// Define if queue should process frames if all inputs are ready.
    ahead_of_time_processing: bool,

    /// Defines behavior when event is scheduled too late:
    /// true - Event will be executed immediately.
    /// false - Event will be discarded.
    run_late_scheduled_events: bool,

    start_sender: Mutex<Option<Sender<QueueStartEvent>>>,
    scheduled_event_sender: Sender<ScheduledEvent>,
}

pub(super) struct QueueVideoOutput {
    pub(super) pts: Duration,
    pub(super) frames: HashMap<InputId, PipelineEvent<Frame>>,
}

impl From<QueueVideoOutput> for FrameSet<InputId> {
    fn from(value: QueueVideoOutput) -> Self {
        Self {
            frames: value
                .frames
                .into_iter()
                .filter_map(|(key, value)| match value {
                    PipelineEvent::Data(data) => Some((key, data)),
                    PipelineEvent::EOS => None,
                })
                .collect(),
            pts: value.pts,
        }
    }
}

pub(super) struct QueueAudioOutput {
    pub samples: HashMap<InputId, PipelineEvent<Vec<InputSamples>>>,
    pub start_pts: Duration,
    pub end_pts: Duration,
}

impl From<QueueAudioOutput> for InputSamplesSet {
    fn from(value: QueueAudioOutput) -> Self {
        Self {
            samples: value
                .samples
                .into_iter()
                .filter_map(|(key, value)| match value {
                    PipelineEvent::Data(data) => Some((key, data)),
                    PipelineEvent::EOS => None,
                })
                .collect(),
            start_pts: value.start_pts,
            end_pts: value.end_pts,
        }
    }
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
    pub run_late_scheduled_events: bool,
}

pub struct ScheduledEvent {
    pts: Duration,
    callback: Box<dyn FnOnce() + Send>,
}

pub enum PipelineEvent<T> {
    Data(T),
    EOS,
}

impl<T: Clone> Clone for PipelineEvent<T> {
    fn clone(&self) -> Self {
        match self {
            PipelineEvent::Data(data) => PipelineEvent::Data(data.clone()),
            PipelineEvent::EOS => PipelineEvent::EOS,
        }
    }
}

impl Queue {
    pub fn new(opts: QueueOptions) -> Arc<Self> {
        let (queue_start_sender, queue_start_receiver) = bounded(0);
        let (scheduled_event_sender, scheduled_event_receiver) = bounded(0);
        let buffer_duration = DEFAULT_BUFFER_DURATION;
        let queue = Arc::new(Queue {
            video_queue: Mutex::new(VideoQueue::new(buffer_duration)),
            output_framerate: opts.output_framerate,

            audio_queue: Mutex::new(AudioQueue::new(buffer_duration)),
            audio_chunk_duration: DEFAULT_AUDIO_CHUNK_DURATION,

            scheduled_event_sender,
            start_sender: Mutex::new(Some(queue_start_sender)),
            ahead_of_time_processing: opts.ahead_of_time_processing,
            run_late_scheduled_events: opts.run_late_scheduled_events,
        });

        QueueThread::new(
            queue.clone(),
            queue_start_receiver,
            scheduled_event_receiver,
        )
        .spawn();

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

    pub(super) fn start(
        self: &Arc<Self>,
        video_sender: Sender<QueueVideoOutput>,
        audio_sender: Sender<QueueAudioOutput>,
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

    pub fn schedule_event(&self, pts: Duration, callback: Box<dyn FnOnce() + Send>) {
        self.scheduled_event_sender
            .send(ScheduledEvent { pts, callback })
            .unwrap();
    }
}

impl PartialOrd for ScheduledEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl cmp::Eq for ScheduledEvent {}

impl cmp::PartialEq for ScheduledEvent {
    fn eq(&self, other: &Self) -> bool {
        self.pts.eq(&other.pts) && std::ptr::eq(&self.callback, &other.callback)
    }
}

impl Ord for ScheduledEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        // Invert duration compare to make heap return smallest values
        match self.pts.cmp(&other.pts) {
            Ordering::Less => Ordering::Greater,
            Ordering::Equal => Ordering::Equal,
            Ordering::Greater => Ordering::Less,
        }
    }
}

impl Debug for ScheduledEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScheduledEvent")
            .field("pts", &self.pts)
            .field("callback", &"<callback>".to_string())
            .finish()
    }
}
