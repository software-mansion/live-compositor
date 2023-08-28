// use compositor_common::scene::{builtin_transformations::TextureLayout, Resolution, ShaderParam};

// pub(super) struct FixedPositionLayoutParams {
//     top: ShaderParam,
//     left: ShaderParam,
//     rotation: ShaderParam,
//     padding: ShaderParam,
// }

// impl FixedPositionLayoutParams {
//     fn new(texture_layout: TextureLayout, output_resolution: Resolution) -> Self {
//         Self {
//             top: ShaderParam::I32(texture_layout.top.pixels(output_resolution.height as u32)),
//             left: ShaderParam::I32(texture_layout.left.pixels(output_resolution.width as u32)),
//             rotation: ShaderParam::I32(texture_layout.rotation.0),
//             padding: ShaderParam::I32(0),
//         }
//     }
// }
