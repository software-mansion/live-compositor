use std::cmp::min;

use compositor_common::{scene::Resolution, util::Coord};

#[derive(Debug)]
pub struct CornersRoundingParams {
    // pixels
    border_radius: f32,
}

impl CornersRoundingParams {
    pub fn new(border_radius: Coord, input_resolutions: &[Option<Resolution>]) -> Self {
        let Some(Some(input_resolution)) = input_resolutions.first() else {
            return Self {border_radius: 0.0};
        };

        let pixels_border_radius = match border_radius {
            Coord::Pixel(pixels) => pixels as f32,
            Coord::Percent(percent) => {
                min(input_resolution.width, input_resolution.height) as f32 * percent as f32
            }
        };

        Self {
            border_radius: pixels_border_radius,
        }
    }

    pub fn shader_buffer_content(&self) -> bytes::Bytes {
        bytes::Bytes::copy_from_slice(&self.border_radius.to_le_bytes())
    }
}
