use compositor_common::scene::builtin_transformations::MirrorMode;

pub trait MirrorModeExt {
    fn shader_buffer_content(&self) -> bytes::Bytes;
}

impl MirrorModeExt for MirrorMode {
    fn shader_buffer_content(&self) -> bytes::Bytes {
        match self {
            MirrorMode::Horizontal => bytes::Bytes::copy_from_slice(&0_u32.to_le_bytes()),
            MirrorMode::Vertical => bytes::Bytes::copy_from_slice(&1_u32.to_le_bytes()),
            MirrorMode::HorizontalAndVertical => {
                bytes::Bytes::copy_from_slice(&2_u32.to_le_bytes())
            }
        }
    }
}
