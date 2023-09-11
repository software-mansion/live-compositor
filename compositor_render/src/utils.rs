use compositor_common::util::colors::RGBAColor;

pub(crate) fn rgba_to_wgpu_color(rgba_color: &RGBAColor) -> wgpu::Color {
    wgpu::Color {
        r: rgba_color.0 as f64 / 255.0,
        g: rgba_color.1 as f64 / 255.0,
        b: rgba_color.2 as f64 / 255.0,
        a: rgba_color.3 as f64 / 255.0,
    }
}
