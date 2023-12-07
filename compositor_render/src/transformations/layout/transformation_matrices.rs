use compositor_common::scene::Resolution;
use nalgebra_glm::{rotate_z, scale, translate, vec3, Mat4, Vec3};

use super::RenderLayout;

impl RenderLayout {
    /// Returns matrix that transforms input plane vertices
    /// (located in corners of clip space), to final position
    pub(super) fn vertices_transformation_matrix(&self, output_resolution: &Resolution) -> Mat4 {
        vertices_transformation_matrix(
            &Position {
                top: self.top,
                left: self.left,
                width: self.width,
                height: self.height,
                rotation_degrees: self.rotation_degrees,
            },
            output_resolution,
        )
    }

    pub(super) fn texture_coords_transformation_matrix(
        &self,
        input_resolution: &Option<Resolution>,
    ) -> Mat4 {
        let Some(input_resolution) = input_resolution else {
            return Mat4::identity();
        };

        match self.content {
            super::RenderLayoutContent::Color(_) => Mat4::identity(),
            super::RenderLayoutContent::ChildNode { ref crop, .. } => {
                let x_scale = crop.width / input_resolution.width as f32;
                let y_scale = crop.height / input_resolution.height as f32;

                let x_translate = crop.left / input_resolution.width as f32;
                let y_translate = crop.top / input_resolution.height as f32;

                let mut transform_texture_matrix = Mat4::identity();
                transform_texture_matrix = translate(
                    &transform_texture_matrix,
                    &vec3(x_translate, y_translate, 0.0),
                );
                transform_texture_matrix =
                    scale(&transform_texture_matrix, &vec3(x_scale, y_scale, 1.0));

                transform_texture_matrix
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct Position {
    pub(crate) top: f32,
    pub(crate) left: f32,
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) rotation_degrees: f32,
}

pub(crate) fn vertices_transformation_matrix(
    position: &Position,
    output_resolution: &Resolution,
) -> Mat4 {
    /// Calculates translation vector from origin to middle of cropped layout box
    /// in ([-output_width / 2, output_width / 2], [-output_height / 2, output_height / 2])
    /// coordinate system
    fn translation_to_final_position(position: &Position, output_resolution: &Resolution) -> Vec3 {
        let left_border_x = -(output_resolution.width as f32 / 2.0);
        let distance_left_to_middle = position.left + (position.width / 2.0);

        let top_border_y = output_resolution.height as f32 / 2.0;
        let top_to_middle = position.top + (position.height / 2.0);
        vec3(
            left_border_x + distance_left_to_middle,
            top_border_y - top_to_middle,
            0.0,
        )
    }

    let mut transform_position_matrix = Mat4::identity();

    let x_scale_to_pixels = output_resolution.width as f32 / 2.0;
    let y_scale_to_pixels = output_resolution.height as f32 / 2.0;

    let x_scale_to_clip_space = 1.0 / x_scale_to_pixels;
    let y_scale_to_clip_space = 1.0 / y_scale_to_pixels;
    transform_position_matrix = scale(
        &transform_position_matrix,
        &vec3(x_scale_to_clip_space, y_scale_to_clip_space, 1.0),
    );

    transform_position_matrix = translate(
        &transform_position_matrix,
        &translation_to_final_position(position, output_resolution),
    );

    transform_position_matrix = rotate_z(
        &transform_position_matrix,
        position.rotation_degrees.to_radians(),
    );

    let x_scale = position.width / output_resolution.width as f32;
    let y_scale = position.height / output_resolution.height as f32;
    transform_position_matrix = scale(
        &transform_position_matrix,
        &vec3(
            x_scale_to_pixels * x_scale,
            y_scale_to_pixels * y_scale,
            1.0,
        ),
    );

    transform_position_matrix
}
