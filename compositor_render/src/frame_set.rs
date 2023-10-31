use std::{collections::HashMap, sync::Arc, time::Duration};

use compositor_common::Frame;

#[derive(Debug)]
pub struct FrameSet<Id>
where
    Id: From<Arc<str>>,
{
    pub frames: HashMap<Id, Frame>,
    pub pts: Duration,
}

impl<Id> FrameSet<Id>
where
    Id: From<Arc<str>>,
{
    pub fn new(pts: Duration) -> Self {
        FrameSet {
            frames: HashMap::new(),
            pts,
        }
    }
}
