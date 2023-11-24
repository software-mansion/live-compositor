use std::time::Duration;

pub mod error;
pub mod frame;
pub mod renderer_spec;
pub mod scene;
pub mod util;

pub type Frame = frame::Frame;

#[derive(Debug, Clone, Copy)]
pub struct Framerate {
    pub num: u32,
    pub den: u32,
}

impl Framerate {
    pub fn get_interval_duration(self) -> Duration {
        Duration::from_nanos(1_000_000_000u64 * self.den as u64 / self.num as u64)
    }
}
