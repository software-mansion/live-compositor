use crate::scene::Resolution;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Frame {
    pub data: YuvData,
    pub resolution: Resolution,
    pub pts: i64,
}

#[derive(Debug, Clone)]
pub struct YuvData {
    pub y_plane: bytes::Bytes,
    pub u_plane: bytes::Bytes,
    pub v_plane: bytes::Bytes,
}
