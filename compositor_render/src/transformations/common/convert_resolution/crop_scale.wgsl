struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

struct CommonParams {
    time: f32,
    textures_count: u32,
    output_texture_size: vec2<u32>,
}

@group(0) @binding(0) var textures: binding_array<texture_2d<f32>, 16>;
@group(2) @binding(0) var sampler_: sampler;

fn translation_matrix(x_translation: f32, y_translation: f32) -> mat3x3<f32> {
    let col1 = vec3<f32>(1.0, 0.0, 0.0);
    let col2 = vec3<f32>(0.0, 1.0, 0.0);
    let col3 = vec3<f32>(x_translation, y_translation, 1.0);

    return mat3x3<f32>(col1, col2, col3);
}

fn scale_matrix(x_scale: f32, y_scale: f32) -> mat3x3<f32> {
    let col1 = vec3<f32>(x_scale, 0.0, 0.0);
    let col2 = vec3<f32>(0.0, y_scale, 0.0);
    let col3 = vec3<f32>(0.0, 0.0, 1.0);
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let input_texture_size: vec2<u32> = textureDimensions(textures[0]);
    let output_texture_size: vec2<u32> = textureDimensions(textures[0]);


    output.position = vec4(input.position, 1.0);
    output.tex_coords = input.tex_coords;

    return output;
}

var<push_constant> common_params: CommonParams;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(textures[0], sampler_, input.tex_coords);
}

