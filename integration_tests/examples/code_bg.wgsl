// 0. background PNG
// 1. Code 

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

struct BaseShaderParameters {
    plane_id: i32,
    time: f32,
    output_resolution: vec2<u32>,
    texture_count: u32,
}

@group(0) @binding(0) var textures: binding_array<texture_2d<f32>, 16>;
@group(2) @binding(0) var sampler_: sampler;

var<push_constant> base_params: BaseShaderParameters;

struct IsInCorner {
    left_border: f32,
    right_border: f32,
    top_border: f32,
    bot_border: f32,
}

fn get_nearest_inner_corner_coords_in_pixels(
    is_on_edge: IsInCorner,
    input_width: f32,
    input_height: f32, 
    border_radius: f32
) -> vec2<f32> {
    let x = is_on_edge.left_border * border_radius + is_on_edge.right_border * (input_width - border_radius);
    let y = is_on_edge.top_border * border_radius + is_on_edge.bot_border * (input_height - border_radius);

    return vec2(x, y);
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    output.position = vec4(input.position, 1.0);
    output.tex_coords = input.tex_coords;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let sample = textureSample(textures[0], sampler_, input.tex_coords);
    let bg_color = textureSample(textures[1], sampler_, vec2(0.01, 0.01));

    if (sample.a < 0.1) {
        return vec4(bg_color.xyz, 1.0);
    } else {
        return sample;
    }
}
