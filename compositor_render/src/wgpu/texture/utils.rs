use crate::Resolution;

pub(crate) fn pad_to_256(value: u32) -> u32 {
    value + (256 - (value % 256))
}

pub fn texture_size_to_resolution(size: &wgpu::Extent3d) -> Resolution {
    Resolution {
        width: size.width as usize,
        height: size.height as usize,
    }
}
