use std::time::Duration;

pub mod frame;
pub mod scene;

pub type Frame = frame::Frame;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InputId(pub u32);

/// TODO: This should be a rational.
#[derive(Debug, Clone, Copy)]
pub struct Framerate(pub u32);

impl Framerate {
    pub fn get_interval_duration(self) -> Duration {
        Duration::from_nanos((1_000_000_000 / self.0).into())
    }
}
