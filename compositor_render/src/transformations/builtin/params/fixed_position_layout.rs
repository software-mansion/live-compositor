use compositor_common::scene::{builtin_transformations::TextureLayout, Resolution};
use nalgebra_glm::{rotate, scale, translate, vec3, Mat4};

pub struct FixedPositionLayoutParams {
    transformation_matrices: Vec<Mat4>,
    background_color: wgpu::Color,
}

impl FixedPositionLayoutParams {
    pub fn new(
        texture_layouts: Vec<TextureLayout>,
        background_color: wgpu::Color,
        input_resolutions: Vec<Option<Resolution>>,
        output_resolution: Resolution,
    ) -> Self {
        let transformation_matrices: Vec<Mat4> = texture_layouts
            .iter()
            .zip(input_resolutions.iter())
            .map(|(texture_layout, input_resolution)| {
                Self::transformation_matrix(texture_layout, input_resolution, output_resolution)
            })
            .collect();

        Self {
            transformation_matrices,
            background_color,
        }
    }

    fn transformation_matrix(
        layout: &TextureLayout,
        input_resolution: &Option<Resolution>,
        output_resolution: Resolution,
    ) -> Mat4 {
        let mut transformation_matrix = Mat4::identity();

        let Some(input_resolution) = input_resolution else {
            return transformation_matrix;
        };

        let scale_to_pixels_x = output_resolution.width as f32 / 2.0;
        let scale_to_pixels_y = output_resolution.height as f32 / 2.0;

        let x_scale = input_resolution.width as f32 / output_resolution.width as f32;
        let y_scale = input_resolution.height as f32 / input_resolution.height as f32;
        // All operations in reverse order, due to matrix multiplication order (read from bottom)

        // Scale back to clip coords
        transformation_matrix = scale(
            &transformation_matrix,
            &vec3(1.0 / scale_to_pixels_x, 1.0 / scale_to_pixels_y, 1.0),
        );

        // Translate to final position
        // This is in different coords, but since it's translation in pixels, it doesn't matter
        let top_left_corner_after_scale = (
            -(output_resolution.width as f32) / 2.0 * x_scale,
            -(output_resolution.height as f32) / 2.0 * y_scale,
        );
        let top_left_corner_final = (
            layout.left.pixels(output_resolution.width as u32) as f32,
            // minus, because user provides pixels from top and we need pixels from bottom
            output_resolution.height as f32
                - layout.top.pixels(output_resolution.height as u32) as f32,
        );

        transformation_matrix = translate(
            &transformation_matrix,
            &vec3(
                top_left_corner_final.0 - top_left_corner_after_scale.0,
                top_left_corner_final.1 - top_left_corner_after_scale.1,
                0.0,
            ),
        );

        // Rotate
        transformation_matrix = rotate(
            &transformation_matrix,
            (layout.rotation.0 as f32).to_radians(),
            &vec3(0.0, 0.0, 1.0),
        );

        // Scale to texture size
        transformation_matrix = scale(&transformation_matrix, &vec3(x_scale, y_scale, 1.0));

        // Scale up to resolution space
        // ([-output_width / 2, output_width / 2], [-output_height / 2, output_height /2])
        transformation_matrix = scale(
            &transformation_matrix,
            &vec3(scale_to_pixels_x, scale_to_pixels_y, 1.0),
        );

        transformation_matrix
    }
}
