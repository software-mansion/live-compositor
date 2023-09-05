use compositor_common::{renderer_spec::FallbackStrategy, util::RGBAColor};

use crate::renderer::texture::NodeTexture;

pub(crate) fn rgba_to_wgpu_color(rgba_color: &RGBAColor) -> wgpu::Color {
    wgpu::Color {
        r: rgba_color.0 as f64 / 255.0,
        g: rgba_color.1 as f64 / 255.0,
        b: rgba_color.2 as f64 / 255.0,
        a: rgba_color.3 as f64 / 255.0,
    }
}

pub(crate) fn does_fallback(
    fallback_strategy: &FallbackStrategy,
    input_node_textures: &[&NodeTexture],
) -> bool {
    match fallback_strategy {
        FallbackStrategy::NeverFallback => false,
        FallbackStrategy::FallbackIfAllInputsMissing => input_node_textures
            .iter()
            .all(|node_texture| node_texture.is_empty()),
        FallbackStrategy::FallbackIfAnyInputsMissing => input_node_textures
            .iter()
            .any(|node_texture| node_texture.is_empty()),
        FallbackStrategy::FallbackIfOnlyInputMissing => {
            input_node_textures.len() == 1
                && input_node_textures
                    .first()
                    .map_or(false, |node_texture| node_texture.is_empty())
        }
    }
}
