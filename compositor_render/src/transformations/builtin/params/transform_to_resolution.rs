use compositor_common::scene::Resolution;
use nalgebra_glm::{scaling, vec3, Mat4};

use crate::transformations::builtin::utils::mat4_to_bytes;

#[derive(Debug, Default)]
pub struct FitParams {
    pub scale_matrix: Mat4,
}

impl FitParams {
    /// This transformation preserves the input texture ratio.
    ///
    /// If the input ratio is larger than the output ratio, the texture is scaled,
    /// such that input width = output width. Then:
    /// scale_factor_pixels = output_width / input_width
    /// Using clip space coords ([-1, 1] range in both axis):
    /// scale_factor_x_clip_space = 1.0 (input x coords are already fitted)
    /// scale_factor_y_clip_space = scale_factor_pixels * input_height / output_height =
    /// = (output_width * input_height) / (output_height * input_width)
    /// = output_ratio / input_ratio
    ///
    /// If the output ratio is larger, then the texture is scaled up,
    /// such that input_height = output_height.
    /// Analogously:
    /// scale_factor_x_clip_space = input_ratio / output_ratio
    /// scale_factor_y_clip_space = 1.0 (input y coords are already fitted)
    pub fn new(input_resolution: Resolution, output_resolution: Resolution) -> Self {
        let input_ratio = input_resolution.ratio();
        let output_ratio = output_resolution.ratio();

        let (x_scale, y_scale) = if input_ratio >= output_ratio {
            (1.0, output_ratio / input_ratio)
        } else {
            (input_ratio / output_ratio, 1.0)
        };

        Self {
            scale_matrix: scaling(&vec3(x_scale, y_scale, 1.0)),
        }
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
