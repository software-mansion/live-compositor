use std::time::Duration;

use crate::scene::Resolution;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Frame {
    pub data: YuvData,
    pub resolution: Resolution,
    pub pts: Duration,
}

#[derive(Debug, Clone)]
pub struct YuvData {
    pub y_plane: bytes::Bytes,
    pub u_plane: bytes::Bytes,
    pub v_plane: bytes::Bytes,
}
