struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

struct Layout {
    vertices_transformation: mat4x4<f32>,
    texture_coord_transformation: mat4x4<f32>,
    color: vec4<f32>, // used only when is_texture == 0
    is_texture: u32, // 0 -> color, 1 -> texture
}

struct RoundedCorner {
    // in pixels of output
    center: vec2<f32>,
    // in pixels of output
    radius: f32,
    // 0 -> top-left, 1 -> top-right, 2 -> bottom-right, 3 -> bottom-left
    direction: u32,
}

struct LayoutInfo {
    layout_id: u32,
    rounded_corners_count: u32,
}

@group(0) @binding(0) var texture: texture_2d<f32>;
// Uniform changing per render pass
@group(1) @binding(0) var<uniform> layouts: array<Layout, 100>;
// Uniforms changing per plane render (draw call)
@group(2) @binding(0) var<uniform> rounded_corners: array<RoundedCorner, 100>;
@group(2) @binding(1) var<uniform> layout_info: LayoutInfo;

@group(3) @binding(0) var sampler_: sampler;


@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    let layout_id: u32 = layout_info.layout_id;
    let vertices_transformation_matrix: mat4x4<f32> = layouts[layout_id].vertices_transformation;
    let texture_coord_transformation_matrix: mat4x4<f32> = layouts[layout_id].texture_coord_transformation;

    output.position = vec4(input.position, 1.0) * vertices_transformation_matrix;
    output.tex_coords = (vec4(input.tex_coords, 0.0, 1.0) * texture_coord_transformation_matrix).xy;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let layout_id: u32 = layout_info.layout_id;
    let current_layout = layouts[layout_id];

    // sampling can't be conditional, so in case of plane_id == -1
    // sample textures[0], but ignore the result.
    if (current_layout.is_texture == 0u) {
        return current_layout.color;
    }
    // clamp transparent, when crop > input texture
    let is_inside: f32 = round(f32(input.tex_coords.x < 1.0 && input.tex_coords.x > 0.0 && input.tex_coords.y > 0.0 && input.tex_coords.y < 1.0));

    let alpha: f32 = cornerns_rounding_alpha(input.tex_coords);
    let sample = textureSample(texture, sampler_, input.tex_coords);

    return is_inside * vec4(sample.xyz, sample.w * alpha);
}

// position in pixels of input texture
fn cornerns_rounding_alpha(position: vec2<f32>) -> f32 {
    let texture_res = textureDimensions(texture);
    let position_pixels = vec2<f32>(
        position.x * f32(texture_res.x),
        (1.0 - position.y) * f32(texture_res.y) // flip y axis, as texture coords go from (0.0, 0.0) - top left to (1.0, 1.0) - bottom right
    );

    var alpha: f32 = 1.0;
    for (var i: u32 = 0u; i < layout_info.rounded_corners_count; i = i + 1u) {
        let corner: RoundedCorner = rounded_corners[i];
        let distance: f32 = length(position_pixels - corner.center);

        let is_left = position_pixels.x < corner.center.x;
        let is_top = position_pixels.y > corner.center.y;

        let anti_aliasing_pixels = 1.5;

        let corner_aplha: f32 = smoothstep(corner.radius + anti_aliasing_pixels, corner.radius - anti_aliasing_pixels, distance);

        switch corner.direction {
            // Top left
            case 0u: {
                if (is_left && is_top) {
                    alpha = min(alpha, corner_aplha);
                }
            }
            // Top right
            case 1u: {
                if (!is_left && is_top) {
                    alpha = min(alpha, corner_aplha);
                }
            }
            // Bottom right
            case 2u: {
                if (!is_left && !is_top) {
                    alpha = min(alpha, corner_aplha);
                }
            }
            // Bottom left
            case 3u: {
                if (is_left && !is_top) {
                    alpha = min(alpha, corner_aplha);
                }
            }
            default: {}
        }
    }
    return alpha;
}
