use compositor_common::scene::Resolution;

#[derive(Debug, Default)]
pub struct FitParams {
    x_scale: f32,
    y_scale: f32,
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
    /// = scale_factor_y_clip_space = output_ratio / input_ratio
    ///
    /// If the output ratio is larger, then the texture is scaled up,
    /// such that input_height = output_height.
    /// Analogously:
    /// scale_factor_x_clip_space = input_ratio / output_ratio
    /// scale_factor_y_clip_space = 1.0 (input y coords are already fitted)
    pub fn new(input_resolution: Resolution, output_resolution: Resolution) -> Self {
        let input_ratio = input_resolution.ratio();
        let output_ratio = output_resolution.ratio();

        if input_ratio >= output_ratio {
            Self {
                x_scale: 1.0,
                y_scale: output_ratio / input_ratio,
            }
        } else {
            Self {
                x_scale: input_ratio / output_ratio,
                y_scale: 1.0,
            }
        }
    }

    pub fn shader_buffer_content(&self) -> bytes::Bytes {
        let mut bytes = bytes::BytesMut::new();
        bytes.extend_from_slice(&self.x_scale.to_le_bytes());
        bytes.extend_from_slice(&self.y_scale.to_le_bytes());

        bytes.freeze()
    }
}

#[derive(Debug, Default)]
pub struct FillParams {
    x_scale: f32,
    y_scale: f32,
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

        if input_ratio >= output_ratio {
            Self {
                x_scale: input_ratio / output_ratio,
                y_scale: 1.0,
            }
        } else {
            Self {
                x_scale: 1.0,
                y_scale: output_ratio / input_ratio,
            }
        }
    }

    pub fn shader_buffer_content(&self) -> bytes::Bytes {
        let mut bytes = bytes::BytesMut::new();
        bytes.extend_from_slice(&self.x_scale.to_le_bytes());
        bytes.extend_from_slice(&self.y_scale.to_le_bytes());

        bytes.freeze()
    }
}
