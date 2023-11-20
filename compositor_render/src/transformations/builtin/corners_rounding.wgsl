struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) texture_id: i32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) @interpolate(flat) texture_id: i32,
}

struct CommonShaderParameters {
    time: f32,
    texture_count: u32,
    output_resolution: vec2<u32>,
}

struct CornersRoudningParams {
    border_radius: f32,
}

var<push_constant> common_params: CommonShaderParameters;

@group(0) @binding(0) var textures: binding_array<texture_2d<f32>, 16>;
@group(1) @binding(0) var<uniform> corners_rounding_params: CornersRoudningParams;
@group(2) @binding(0) var sampler_: sampler;


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
    // Firstly calculates, whether the pixel is in the square in one of the video corners,
    // then calculates the distance to the center of the circle located in corner of the video
    // and applies the smoothstep functon to the alpha value of the pixel.

    let border_radius: f32 = corners_rounding_params.border_radius;
    let input_resolution: vec2<u32> = textureDimensions(textures[0]);
    let input_width: f32 = f32(input_resolution.x);
    let input_height: f32 = f32(input_resolution.y);

    var is_on_edge: IsInCorner;

    is_on_edge.left_border = f32((input.tex_coords.x * input_width) < border_radius);
    is_on_edge.right_border = f32((input.tex_coords.x * input_width) > input_width - border_radius);
    is_on_edge.top_border = f32((input.tex_coords.y * input_height) < border_radius);
    is_on_edge.bot_border = f32((input.tex_coords.y * input_height) > input_height - border_radius);

    let is_in_corner = max(is_on_edge.left_border, is_on_edge.right_border) * max(is_on_edge.top_border, is_on_edge.bot_border);
    let color = textureSample(textures[0], sampler_, input.tex_coords);

    let corner_coords = get_nearest_inner_corner_coords_in_pixels(
        is_on_edge,
        input_width,
        input_height,
        border_radius
    );

    let d = distance(input.tex_coords * vec2(input_width, input_height), corner_coords);

    let anti_aliasing_pixels = 1.5;

    let alpha = smoothstep(border_radius + anti_aliasing_pixels, border_radius - anti_aliasing_pixels, d);

    return vec4(color.xyz, is_in_corner * alpha + (1.0 - is_in_corner));
}
