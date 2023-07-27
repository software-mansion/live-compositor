use std::{collections::HashMap, sync::Arc, time::Duration};

use compositor_common::{Frame, InputId};

#[derive(Debug)]
pub struct InputFrames {
    pub frames: HashMap<InputId, Arc<Frame>>,
    pub pts: Duration,
}

impl InputFrames {
    pub fn new(pts: Duration) -> Self {
        InputFrames {
            frames: HashMap::new(),
            pts,
        }
    }

    pub fn insert_frame(&mut self, input_id: InputId, frame: Arc<Frame>) {
        self.frames.insert(input_id, frame);
    }
}
