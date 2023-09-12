use compositor_common::{
    scene::Resolution,
    util::align::{HorizontalAlign, VerticalAlign},
};
use nalgebra_glm::{scaling, vec3, Mat4};

use crate::transformations::builtin::{box_layout::BoxLayout, utils::mat4_to_bytes};

#[derive(Debug, Default)]
pub struct FitParams {
    pub scale_matrix: Mat4,
}

impl FitParams {
    pub fn new(
        input_resolution: Resolution,
        output_resolution: Resolution,
        x_align: HorizontalAlign,
        y_align: VerticalAlign,
    ) -> Self {
        let scale_matrix = BoxLayout {
            top_left_corner: (0.0, 0.0),
            width: output_resolution.width as f32,
            height: output_resolution.height as f32,
            rotation_degrees: 0.0,
        }
        .fit(input_resolution, x_align, y_align)
        .transformation_matrix(output_resolution);

        Self { scale_matrix }
    }

    pub fn shader_buffer_content(&self) -> bytes::Bytes {
        mat4_to_bytes(&self.scale_matrix)
    }
}

#[derive(Debug, Default)]
pub struct FillParams {
    scale_matrix: Mat4,
}

impl FillParams {
    // This transformation preserves the input texture ratio.
    //
    // If the input ratio is larger than the output ratio, the texture is scaled,
    // such that input height = output height. Then:
    // scale_factor_pixels = output_height / input_height
    // Using clip space coords ([-1, 1] range in both axis):
    // scale_factor_x_clip_space = scale_factor_pixels * input_width / output_width
    // scale_factor_x_clip_space = (output_height * input_width) / (output_width * input_height)
    // scale_factor_x_clip_space = input_ratio / output_ratio
    // scale_factor_y_clip_space = 1.0 (input y coords are already fitted)
    //
    // If the output ratio is larger, then the texture is scaled up,
    // such that input_width = output_width.
    // Analogously:
    // scale_factor_x_clip_space = 1.0 (input x coords are already fitted)
    // scale_factor_y_clip_space = output_ratio / input_ratio
    pub fn new(input_resolution: Resolution, output_resolution: Resolution) -> Self {
        let input_ratio = input_resolution.ratio();
        let output_ratio = output_resolution.ratio();

        let (x_scale, y_scale) = if input_ratio >= output_ratio {
            (input_ratio / output_ratio, 1.0)
        } else {
            (1.0, output_ratio / input_ratio)
        };

        Self {
            scale_matrix: scaling(&vec3(x_scale, y_scale, 1.0)),
        }
    }

    pub fn shader_buffer_content(&self) -> bytes::Bytes {
        mat4_to_bytes(&self.scale_matrix)
    }
}
