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

struct TextureInfo {
     is_website_texture: i32,
     transformation_matrix: mat4x4<f32>,
}

var<push_constant> texture_count: u32;

@group(0) @binding(0) var textures: binding_array<texture_2d<f32>, 16>;
@group(1) @binding(0) var<uniform> textures_info: array<TextureInfo, 16>;
@group(2) @binding(0) var sampler_: sampler;


@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    // 0 inputs case
    if input.texture_id == -1 {
        output.position = vec4<f32>(input.position, 1.0);
        output.tex_coords = input.tex_coords;
        output.texture_id = 0;
        return output;
    }

    let transformation_matrix: mat4x4<f32> = textures_info[input.texture_id].transformation_matrix;

    output.position = vec4(input.position, 1.0) * transformation_matrix;
    output.tex_coords = input.tex_coords;
    output.texture_id = input.texture_id;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {

    let sample = textureSample(textures[input.texture_id], sampler_, input.tex_coords);

    if texture_count == 0u {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    // Website texture uses BGRA.
    // Convert BGRA to RGBA
    if textures_info[input.texture_id].is_website_texture == 1 {
        return vec4<f32>(sample.b, sample.g, sample.r, sample.a);
    }

    return sample;
}
