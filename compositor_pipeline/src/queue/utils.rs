use std::{
    collections::VecDeque,
    mem,
    time::{Duration, Instant},
};

use compositor_render::{AudioSamplesBatch, Frame};

/// InputState handles initial processing for frames/samples that are being
/// queued. For each received frame/sample batch, the `process_new_chunk`
/// method should be called and only elements returned should be used
/// in a queue.
///
/// 1. New input start in `InputState::WaitingForStart`.
/// 2. When `process_new_chunk` is called for the first time it transitions to
///    the Buffering state.
/// 3. Each new call to the `process_new_chunk` is adding frames to the buffer
///    until it reaches a specific size/duration.
/// 4. After buffer reaches a certain size, calculate the offset and switch
///    to the `Ready` state.
/// 5. In `Ready` state `process_new_chunk` is immediately returning frame or sample
///    batch passed with arguments with modified pts.
#[derive(Debug)]
pub(super) struct InputProcessor<Payload: ApplyOffsetExt> {
    buffer_duration: Duration,
    state: InputState<Payload>,
}

#[derive(Debug)]
pub(super) enum InputState<Payload: ApplyOffsetExt> {
    WaitingForStart,
    Buffering {
        buffer: Vec<(Payload, Duration)>,
    },
    Ready {
        /// Offset that needs to applied to convert PTS of input
        /// frames into a time frame where PTS=0 represents first
        /// frame
        offset: Duration,
        /// Moment where input transitioned to a ready state
        start_time: Instant,
    },
}

impl<Payload: ApplyOffsetExt> InputProcessor<Payload> {
    pub(super) fn new(buffer_duration: Duration) -> Self {
        Self {
            buffer_duration,
            state: InputState::WaitingForStart,
        }
    }

    pub(super) fn start_time(&self) -> Option<Instant> {
        match self.state {
            InputState::Ready { start_time, .. } => Some(start_time),
            _ => None,
        }
    }

    pub(super) fn process_new_chunk(
        &mut self,
        mut payload: Payload,
        pts: Duration,
    ) -> VecDeque<Payload> {
        match self.state {
            InputState::WaitingForStart => {
                self.state = InputState::Buffering {
                    buffer: vec![(payload, pts)],
                };
                VecDeque::new()
            }
            InputState::Buffering { ref mut buffer } => {
                buffer.push((payload, pts));
                let first_pts = buffer.first().map(|(_, p)| *p).unwrap_or(Duration::ZERO);
                let last_pts = buffer.last().map(|(_, p)| *p).unwrap_or(Duration::ZERO);
                let buffer_duration = last_pts.saturating_sub(first_pts);

                if buffer_duration < self.buffer_duration {
                    VecDeque::new()
                } else {
                    let offset = first_pts;

                    let chunks = mem::take(buffer)
                        .into_iter()
                        .map(|(mut buffer, _)| {
                            buffer.apply_offset(offset);
                            buffer
                        })
                        .collect();
                    self.state = InputState::Ready {
                        offset,
                        start_time: Instant::now(),
                    };
                    chunks
                }
            }
            InputState::Ready { offset, .. } => {
                payload.apply_offset(offset);
                VecDeque::from([payload])
            }
        }
    }
}

pub(super) trait ApplyOffsetExt {
    fn apply_offset(&mut self, offset: Duration);
}

impl ApplyOffsetExt for Frame {
    fn apply_offset(&mut self, offset: Duration) {
        self.pts = self.pts.saturating_sub(offset)
    }
}

impl ApplyOffsetExt for AudioSamplesBatch {
    fn apply_offset(&mut self, offset: Duration) {
        self.start_pts = self.start_pts.saturating_sub(offset)
    }
}
