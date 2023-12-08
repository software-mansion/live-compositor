pub mod id;
pub mod shader;

pub use id::InputId;
pub use id::OutputId;

pub const MAX_NODE_RESOLUTION: Resolution = Resolution {
    width: 7682,
    height: 4320,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Resolution {
    pub width: usize,
    pub height: usize,
}

impl Resolution {
    pub fn ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }
}
