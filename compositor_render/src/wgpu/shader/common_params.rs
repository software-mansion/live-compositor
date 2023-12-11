use std::time::Duration;

use crate::Resolution;

#[repr(C)]
#[derive(Debug, bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
pub struct CommonShaderParameters {
    time: f32,
    pub texture_count: u32,
    output_resolution: [u32; 2],
}

impl CommonShaderParameters {
    pub fn new(time: Duration, texture_count: u32, output_resolution: Resolution) -> Self {
        Self {
            time: time.as_secs_f32(),
            texture_count,
            output_resolution: [
                output_resolution.width as u32,
                output_resolution.height as u32,
            ],
        }
    }

    pub fn push_constant_size() -> u32 {
        let size = std::mem::size_of::<CommonShaderParameters>() as u32;
        match size % 4 {
            0 => size,
            rest => size + (4 - rest),
        }
    }

    pub fn push_constant(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}
