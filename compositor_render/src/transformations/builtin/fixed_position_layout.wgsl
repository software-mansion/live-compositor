struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) texture_id: u32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) @interpolate(flat) texture_id: u32,
}

struct CommonShaderParameters {
    time: f32,
    textures_count: u32,
    output_resolution: vec2<u32>,
}

struct TextureLayout {
    top: i32,
    left: i32,
    rotation: i32,
    padding: i32, // has to be alligned to 16
}

struct Layouts {
    textures_layouts: array<TextureLayout, 16>
}

var<push_constant> common_params: CommonShaderParameters;

@group(0) @binding(0) var textures: binding_array<texture_2d<f32>, 16>;
@group(1) @binding(0) var<uniform> layouts: Layouts;
@group(2) @binding(0) var sampler_: sampler;

// Lineary interpolates x from [0, x_max] to [-1.0, 1.0] domain
fn to_clip_space_coords(x: f32, x_max: f32) -> f32 {
    return x / x_max * 2.0 - 1.0;
}

fn final_box_position(
    output_texture_width: u32, 
    output_texture_height: u32, 
    texture_layout: TextureLayout
) -> vec3<f32> {
    return vec3<f32>(
        to_clip_space_coords(f32(texture_layout.top), f32(output_texture_width)),
        to_clip_space_coords(f32(texture_layout.top), f32(output_texture_height)),
        0.0
    );
}

fn rotation_matrix(degrees: i32) -> mat3x3<f32> {
    let pi: f32 = 3.14159;
    let radians: f32 = f32(degrees) * pi / 180.0;
    let col1: vec3<f32> = vec3<f32>(1.0, 0.0, 0.0);
    let col2: vec3<f32> = vec3<f32>(0.0, cos(radians), -sin(radians)); 
    let col3: vec3<f32> = vec3<f32>(0.0, sin(radians), cos(radians)); 
    return mat3x3<f32>(col1, col2, col3);
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let input_texture_size: vec2<u32> = textureDimensions(textures[input.texture_id]);
    let texture_layout: TextureLayout = layouts.textures_layouts[input.texture_id];

    let x_scale: f32 = f32(input_texture_size.x) / f32(common_params.output_resolution.x);
    let y_scale: f32 = f32(input_texture_size.y) / f32(common_params.output_resolution.y);

    let scaled_input: vec3<f32> = vec3(input.position.x * x_scale, input.position.y * y_scale, input.position.z);
    let final_position: vec3<f32> = final_box_position(
        common_params.output_resolution.x,
        common_params.output_resolution.y,
        texture_layout
    );

    let translation: vec3<f32> = final_position - scaled_input;
    let rotation_matrix: mat3x3<f32> = rotation_matrix(texture_layout.rotation);
    let output_position = (scaled_input * rotation_matrix) * translation;

    output.position = vec4(output_position, 1.0);
    output.tex_coords = input.tex_coords;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(textures[input.texture_id], sampler_, input.tex_coords);
}
