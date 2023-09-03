use compositor_common::scene::{builtin_transformations::TextureLayout, Resolution};
use nalgebra_glm::{rotate_z, scale, translate, vec3, Mat4};

pub struct FixedPositionLayoutParams {
    transformation_matrices: Vec<Mat4>,
}

impl FixedPositionLayoutParams {
    // TODO take spec
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

    // TODO: explain this witchcraft
    fn transformation_matrix(
        layout: &TextureLayout,
        input_resolution: Option<&Resolution>,
        output_resolution: Resolution,
    ) -> Mat4 {
        let mut transformation_matrix = Mat4::identity();

        let Some(input_resolution) = input_resolution else {
            return transformation_matrix;
        };
        

        let left = layout.left.pixels(output_resolution.width as u32) as f32;
        let top = layout.top.pixels(output_resolution.height as u32) as f32;

        transformation_matrix = scale(
            &transformation_matrix,
            &vec3(
                2.0 / output_resolution.width as f32,
                2.0 / output_resolution.height as f32,
                1.0,
            ),
        );

        transformation_matrix = translate(
            &transformation_matrix,
            &vec3(
                -(output_resolution.width as f32) / 2.0
                    + left
                    + input_resolution.width as f32 / 2.0,
                output_resolution.height as f32 / 2.0 - top - input_resolution.height as f32 / 2.0,
                0.0,
            ),
        );

        transformation_matrix = rotate_z(
            &transformation_matrix,
            (layout.rotation.0 as f32).to_radians(),
        );

        transformation_matrix = scale(
            &transformation_matrix,
            &vec3(
                input_resolution.width as f32 / 2.0,
                input_resolution.height as f32 / 2.0,
                1.0,
            ),
        );

        transformation_matrix
    }

    pub fn shader_buffer_content(&self) -> bytes::Bytes {
        let mut matrices_bytes = bytes::BytesMut::new();
        for matrix in &self.transformation_matrices {
            let colum_based = matrix.transpose();
            for el in &colum_based {
                matrices_bytes.extend_from_slice(&el.to_ne_bytes())
            };
        }

        matrices_bytes.freeze()
    }
}
