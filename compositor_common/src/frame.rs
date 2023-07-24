use std::{collections::HashMap, sync::Arc, time::Duration};

use crate::scene::Resolution;

// Clone is temporary. Testing use only.
#[derive(Debug, Clone)]
pub struct Frame {
    pub data: YuvData,
    pub resolution: Resolution,
    pub pts: Duration,
}

pub type InputID = u32;

/// TODO: This should be a rational.
#[derive(Debug, Clone, Copy)]
pub struct Framerate(pub u32);

impl Framerate {
    pub fn get_interval_duration(self) -> Duration {
        Duration::from_nanos((1_000_000_000 / self.0).into())
    }
}

#[derive(Debug)]
pub struct FramesBatch {
    pub frames: HashMap<InputID, Arc<Frame>>,
    pub pts: Duration,
}

impl FramesBatch {
    pub fn new(pts: Duration) -> Self {
        FramesBatch {
            frames: HashMap::new(),
            pts,
        }
    }

    pub fn insert_frame(&mut self, input_id: InputID, frame: Arc<Frame>) {
        self.frames.insert(input_id, frame);
    }
}

// Clone is temporary. Testing use only.
#[derive(Debug, Clone)]
pub struct YuvData {
    pub y_plane: bytes::Bytes,
    pub u_plane: bytes::Bytes,
    pub v_plane: bytes::Bytes,
}
