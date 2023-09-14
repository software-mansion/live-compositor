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

var<push_constant> common_params: CommonShaderParameters;

@group(0) @binding(0) var textures: binding_array<texture_2d<f32>, 16>;
@group(1) @binding(0) var<uniform> transformation_matrices: array<mat4x4<f32>, 16>;
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
    
    let transformation_matrix: mat4x4<f32> = transformation_matrices[input.texture_id];

    output.position = vec4(input.position, 1.0) * transformation_matrix;
    output.tex_coords = input.tex_coords;
    output.texture_id = input.texture_id;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let sample = textureSample(textures[input.texture_id], sampler_, input.tex_coords);
    
    if common_params.texture_count == 0u {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    return sample;
}
