use compositor_common::scene::Resolution;
use nalgebra_glm::{rotate_z, scale, translate, vec3, Mat4, Vec3};

use super::Layout;

impl Layout {
    /// Returns matrix that transforms input plane vertices
    /// (located in corners of clip space), to final position
    pub(super) fn transformation_matrix(&self, output_resolution: Resolution) -> Mat4 {
        let mut transformation_matrix = Mat4::identity();

        let x_scale_to_pixels = output_resolution.width as f32 / 2.0;
        let y_scale_to_pixels = output_resolution.height as f32 / 2.0;

        let x_scale_to_clip_space = 1.0 / x_scale_to_pixels;
        let y_scale_to_clip_space = 1.0 / y_scale_to_pixels;
        transformation_matrix = scale(
            &transformation_matrix,
            &vec3(x_scale_to_clip_space, y_scale_to_clip_space, 1.0),
        );

        transformation_matrix = translate(
            &transformation_matrix,
            &Self::translation_to_final_position(self, output_resolution),
        );

        transformation_matrix =
            rotate_z(&transformation_matrix, self.rotation_degrees.to_radians());

        let x_scale = self.width / output_resolution.width as f32;
        let y_scale = self.height / output_resolution.height as f32;
        transformation_matrix = scale(
            &transformation_matrix,
            &vec3(
                x_scale_to_pixels * x_scale,
                y_scale_to_pixels * y_scale,
                1.0,
            ),
        );

        transformation_matrix
    }

    /// Calculates translation vector from origin to middle of box
    /// in ([-output_width / 2, output_width / 2], [-output_height / 2, output_height / 2])
    /// coordinate system
    fn translation_to_final_position(&self, output_resolution: Resolution) -> Vec3 {
        vec3(
            -(output_resolution.width as f32 / 2.0) + self.left + (self.width / 2.0),
            (output_resolution.height as f32 / 2.0) - self.top - (self.height / 2.0),
            0.0,
        )
    }
}
