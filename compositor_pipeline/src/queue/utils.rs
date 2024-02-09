use std::{
    collections::VecDeque,
    mem,
    time::{Duration, Instant},
};

use compositor_render::{AudioSamplesBatch, Frame};

use super::DEFAULT_BUFFER_DURATION;

#[derive(Debug)]
pub(super) enum InputState<Payload: ApplyOffsetExt> {
    WaitingForStart,
    Buffering { buffer: Vec<(Payload, Duration)> },
    Ready { offset: chrono::Duration },
}

impl<Payload: ApplyOffsetExt> InputState<Payload> {
    pub(super) fn process_new_chunk(
        &mut self,
        mut payload: Payload,
        pts: Duration,
        clock_start: Instant,
    ) -> VecDeque<Payload> {
        match self {
            InputState::WaitingForStart => {
                *self = InputState::Buffering {
                    buffer: vec![(payload, pts)],
                };
                VecDeque::new()
            }
            InputState::Buffering { ref mut buffer } => {
                buffer.push((payload, pts));
                let first_pts = buffer.first().map(|(_, p)| *p).unwrap_or(Duration::ZERO);
                let last_pts = buffer.last().map(|(_, p)| *p).unwrap_or(Duration::ZERO);
                let buffer_duration = last_pts.saturating_sub(first_pts);

                if buffer_duration < DEFAULT_BUFFER_DURATION {
                    VecDeque::new()
                } else {
                    let offset = clock_start.elapsed().chrono() - first_pts.chrono();

                    let chunks = mem::take(buffer)
                        .into_iter()
                        .map(|(mut buffer, _)| {
                            buffer.apply_offset(offset);
                            buffer
                        })
                        .collect();
                    *self = InputState::Ready { offset };
                    chunks
                }
            }
            InputState::Ready { offset } => {
                payload.apply_offset(*offset);
                VecDeque::from([payload])
            }
        }
    }
}

pub(super) trait ApplyOffsetExt {
    fn apply_offset(&mut self, offset: chrono::Duration);
}

impl ApplyOffsetExt for Frame {
    fn apply_offset(&mut self, offset: chrono::Duration) {
        self.pts = (self.pts.chrono() + offset)
            .to_std()
            .unwrap_or(Duration::ZERO);
    }
}

impl ApplyOffsetExt for AudioSamplesBatch {
    fn apply_offset(&mut self, offset: chrono::Duration) {
        self.start_pts = (self.start_pts.chrono() + offset)
            .to_std()
            .unwrap_or(Duration::ZERO);
    }
}

trait DurationExt {
    fn chrono(self) -> chrono::Duration;
}

impl DurationExt for Duration {
    fn chrono(self) -> chrono::Duration {
        chrono::Duration::from_std(self).unwrap_or(chrono::Duration::max_value())
    }
}
