use std::time::Duration;

pub mod frame;
pub mod scene;
pub mod transformation;
mod validators;

pub type Frame = frame::Frame;
pub type SpecValidationError = validators::SpecValidationError;

/// TODO: This should be a rational.
#[derive(Debug, Clone, Copy)]
pub struct Framerate(pub u32);

impl Framerate {
    pub fn get_interval_duration(self) -> Duration {
        Duration::from_nanos((1_000_000_000 / self.0).into())
    }
}
