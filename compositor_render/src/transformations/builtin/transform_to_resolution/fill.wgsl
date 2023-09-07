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

struct FillParams {
    x_scale: f32,
    y_scale: f32
}

struct CommonShaderParameters {
    time: f32,
    textures_count: u32,
    output_resolution: vec2<u32>,
}

var<push_constant> common_params: CommonShaderParameters;

@group(0) @binding(0) var textures: binding_array<texture_2d<f32>, 16>;
@group(1) @binding(0) var<uniform> fill_params: FillParams;
@group(2) @binding(0) var sampler_: sampler;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    output.position = vec4<f32>(
        input.position.x * fill_params.x_scale, 
        input.position.y * fill_params.y_scale, 
        input.position.z, 1.0
    );
    output.tex_coords = input.tex_coords;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(textures[0], sampler_, input.tex_coords);
}
