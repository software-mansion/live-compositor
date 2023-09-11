use compositor_common::scene::{builtin_transformations::TextureLayout, Resolution};
use log::error;
use nalgebra_glm::Mat4;

use crate::transformations::builtin::{box_layout::BoxLayout, utils::mat4_to_bytes};

#[derive(Debug)]
pub struct FixedPositionLayoutParams {
    transformation_matrices: Vec<Mat4>,
}

impl FixedPositionLayoutParams {
    pub fn new(
        texture_layouts: &[TextureLayout],
        input_resolutions: &[Option<Resolution>],
        output_resolution: Resolution,
    ) -> Self {
        let transformation_matrices: Vec<Mat4> = texture_layouts
            .iter()
            .zip(input_resolutions.iter())
            .map(|(texture_layout, &input_resolution)| {
                Self::transformation_matrix(
                    texture_layout,
                    input_resolution.as_ref(),
                    output_resolution,
                )
            })
            .collect();

        Self {
            transformation_matrices,
        }
    }

    fn spec_to_top_left_coords(
        layout: &TextureLayout,
        input_resolution: &Resolution,
        output_resolution: &Resolution,
    ) -> (f32, f32) {
        let top = match layout {
            TextureLayout { top: Some(top), .. } => top.pixels(output_resolution.height as u32),
            TextureLayout {
                bottom: Some(bottom),
                ..
            } => {
                output_resolution.height as i32
                    - input_resolution.height as i32
                    - bottom.pixels(output_resolution.height as u32)
            }
            _ => {
                error!("Invalid specs in fixed_position_layout: {:?}", layout);
                0
            }
        };
        let left = match layout {
            TextureLayout {
                left: Some(left), ..
            } => left.pixels(output_resolution.width as u32),
            TextureLayout {
                right: Some(right), ..
            } => {
                output_resolution.width as i32
                    - input_resolution.width as i32
                    - right.pixels(output_resolution.width as u32)
            }
            _ => {
                error!("Invalid specs in fixed_position_layout: {:?}", layout);
                0
            }
        };

        (top as f32, left as f32)
    }

    pub fn transformation_matrix(
        layout: &TextureLayout,
        input_resolution: Option<&Resolution>,
        output_resolution: Resolution,
    ) -> Mat4 {
        let Some(input_resolution) = input_resolution else {
            return Mat4::identity();
        };

        let (top, left) =
            Self::spec_to_top_left_coords(layout, input_resolution, &output_resolution);

        let box_layout = BoxLayout {
            top_left_corner: (left, top),
            width: input_resolution.width as f32,
            height: input_resolution.height as f32,
            rotation_degrees: layout.rotation.0 as f32,
        };

        box_layout.transformation_matrix(output_resolution)
    }

    pub fn shader_buffer_content(&self) -> bytes::Bytes {
        let mut matrices_bytes = bytes::BytesMut::new();
        for matrix in &self.transformation_matrices {
            let matrix_bytes = mat4_to_bytes(matrix);
            matrices_bytes.extend(matrix_bytes);
        }

        matrices_bytes.freeze()
    }
}
