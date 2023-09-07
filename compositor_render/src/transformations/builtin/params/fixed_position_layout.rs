use compositor_common::scene::{builtin_transformations::TextureLayout, Resolution};
use log::error;
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
        let mut transformation_matrix = Mat4::identity();

        let Some(input_resolution) = input_resolution else {
            return transformation_matrix;
        };

        let (top, left) =
            Self::spec_to_top_left_coords(layout, input_resolution, &output_resolution);

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

        // Left bound of pixel coords is -output_resolution.width / 2.0 (coords of left corners)
        // `left` is a shift in x axis
        // center of texture is input_resolution.width / 2.0 away from left corners of input texture
        let input_center_left =
            -(output_resolution.width as f32) / 2.0 + left + input_resolution.width as f32 / 2.0;

        // Top bound of pixel coords is output_resolution.height / 2.0
        // `top` is shift in y axis (user provided "top" is subtracted from top bound)
        // center of texture is input_resolution.height / 2.0 away from top corners of input texture
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
