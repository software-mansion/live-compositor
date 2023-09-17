use compositor_common::{
    scene::Resolution,
    util::align::{HorizontalAlign, VerticalAlign},
};
use nalgebra_glm::{rotate_z, scale, translate, vec3, Mat4, Vec3};

#[derive(Debug)]
pub struct BoxLayout {
    // pixels in [0, output_resolution] coords
    pub top_left_corner: (f32, f32),
    pub width: f32,
    pub height: f32,
    pub rotation_degrees: f32,
}

impl BoxLayout {
    /// Returns box representing position of input frame fitted into `self`,     
    /// Fitted means scaled to input resolution without cropping or changing aspect ratio
    pub fn fit(
        &self,
        input_resolution: Resolution,
        x_align: HorizontalAlign,
        y_align: VerticalAlign,
    ) -> BoxLayout {
        let input_x_scale = self.width / input_resolution.width as f32;
        let input_y_scale = self.height / input_resolution.height as f32;

        let input_scale = input_x_scale.min(input_y_scale);

        let x_padding = self.width - input_resolution.width as f32 * input_scale;
        let y_padding = self.height - input_resolution.height as f32 * input_scale;

        let left_padding = match x_align {
            HorizontalAlign::Left => 0.0,
            HorizontalAlign::Right => x_padding,
            HorizontalAlign::Center | HorizontalAlign::Justified => x_padding / 2.0,
        };

        let top_padding = match y_align {
            VerticalAlign::Top => 0.0,
            VerticalAlign::Bottom => y_padding,
            VerticalAlign::Center | VerticalAlign::Justified => y_padding / 2.0,
        };

        BoxLayout {
            top_left_corner: (
                self.top_left_corner.0 + left_padding,
                self.top_left_corner.1 + top_padding,
            ),
            width: self.width - x_padding,
            height: self.height - y_padding,
            rotation_degrees: self.rotation_degrees,
        }
    }

    /// Returns matrix that transforms input plane vertices
    /// (located in corners of clip space), to final position
    pub fn transformation_matrix(&self, output_resolution: Resolution) -> Mat4 {
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
            -(output_resolution.width as f32 / 2.0) + self.top_left_corner.0 + (self.width / 2.0),
            (output_resolution.height as f32 / 2.0) - self.top_left_corner.1 - (self.height / 2.0),
            0.0,
        )
    }
}
