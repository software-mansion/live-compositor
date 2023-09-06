use compositor_common::scene::{builtin_transformations::TextureLayout, Resolution};
use nalgebra_glm::{rotate_z, scale, translate, vec3, Mat4};

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

    fn transformation_matrix(
        layout: &TextureLayout,
        input_resolution: Option<&Resolution>,
        output_resolution: Resolution,
    ) -> Mat4 {
        let mut transformation_matrix = Mat4::identity();

        let Some(input_resolution) = input_resolution else {
            return transformation_matrix;
        };

        // All transformations are applied in reverse order, due to matrix multiplication order.

        // 4. Scale back from pixel coords into clip space coords
        transformation_matrix = scale(
            &transformation_matrix,
            &vec3(
                2.0 / output_resolution.width as f32,
                2.0 / output_resolution.height as f32,
                1.0,
            ),
        );

        // 3. Translate input texture into correct place on output texture.
        // Calculates coords of center of input texture on output texture in pixel coords

        let left = layout.left.pixels(output_resolution.width as u32) as f32;
        // Left bound of pixel coords is -output_resolution.width / 2.0 (coords of left corners)
        // `left` is a shift in x axis
        // center of texture is input_resolution.width / 2.0 away from left corners of input texture
        let input_center_left =
            -(output_resolution.width as f32) / 2.0 + left + input_resolution.width as f32 / 2.0;

        // Top bound of pixel coords is output_resolution.height / 2.0
        // `top` is shift in y axis (user provided "top" is subtracted from top bound)
        // center of texture is input_resolution.height / 2.0 away from top corners of input texture
        let top = layout.top.pixels(output_resolution.height as u32) as f32;
        let input_center_top =
            output_resolution.height as f32 / 2.0 - top - input_resolution.height as f32 / 2.0;

        transformation_matrix = translate(
            &transformation_matrix,
            &vec3(input_center_left, input_center_top, 0.0),
        );

        // 2. Rotate - we want to do this before translation,
        // since we want to rotate around middle of input texture

        transformation_matrix = rotate_z(
            &transformation_matrix,
            (layout.rotation.0 as f32).to_radians(),
        );

        // 1. Scale texture to ([-output_resolution.width / 2.0, output_resolution.width /2.0],
        // [-output_resolution.height / 2.0, output_resolution.height /2.0]) coords.
        // We need to scale it to match input resolution ratio, because rotation by degree non divisible
        // by 90 degrees in clip space coords will be non-affine transformation
        // (distorted, edges of the input texture won't be perpendicular) after mapping on output texture

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
            }
        }

        matrices_bytes.freeze()
    }
}
