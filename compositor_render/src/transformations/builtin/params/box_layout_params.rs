use compositor_common::{
    scene::Resolution,
    util::{ContinuousValue, InterpolationState},
};

use crate::transformations::builtin::{box_layout::BoxLayout, utils::mat4_to_bytes};

#[derive(Debug, Clone)]
pub struct BoxLayoutParams {
    pub boxes: Vec<BoxLayout>,
    pub output_resolution: Resolution,
}

impl BoxLayoutParams {
    pub fn shader_buffer_content(&self) -> bytes::Bytes {
        let matrices = self
            .boxes
            .iter()
            .map(|b| b.transformation_matrix(self.output_resolution));

        let mut matrices_bytes = bytes::BytesMut::new();
        for matrix in matrices {
            let matrix_bytes = mat4_to_bytes(&matrix);
            matrices_bytes.extend(matrix_bytes);
        }

        matrices_bytes.freeze()
    }
}

impl ContinuousValue for BoxLayoutParams {
    fn interpolate(start: &Self, end: &Self, state: InterpolationState) -> Self {
        let boxes = start
            .boxes
            .iter()
            .zip(end.boxes.iter())
            .map(|(start, end)| BoxLayout::interpolate(start, end, state))
            .collect();
        Self {
            boxes,
            output_resolution: start.output_resolution,
        }
    }
}
