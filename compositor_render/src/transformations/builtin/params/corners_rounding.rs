#[derive(Debug)]
pub struct CornersRoundingParams {
    border_radius: u32,
}

impl CornersRoundingParams {
    pub fn new(border_radius: u32) -> Self {
        Self { border_radius }
    }

    pub fn shader_buffer_content(&self) -> bytes::Bytes {
        bytes::Bytes::copy_from_slice(&self.border_radius.to_le_bytes())
    }
}
