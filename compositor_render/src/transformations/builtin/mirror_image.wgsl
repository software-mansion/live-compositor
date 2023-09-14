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

struct MirrorParams {
    mode: u32,
}

var<push_constant> common_params: CommonShaderParameters;

@group(0) @binding(0) var textures: binding_array<texture_2d<f32>, 16>;
@group(1) @binding(0) var<uniform> mirror_params: MirrorParams;
@group(2) @binding(0) var sampler_: sampler;


@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    output.position = vec4(input.position, 1.0);
    if mirror_params.mode == 0u {
        output.tex_coords = vec2<f32>(1.0 - input.tex_coords.x, input.tex_coords.y);
    } else if mirror_params.mode == 1u {
        output.tex_coords = vec2<f32>(input.tex_coords.x, 1.0 - input.tex_coords.y);
    } else {
        output.tex_coords = vec2<f32>(1.0 - input.tex_coords.x, 1.0 - input.tex_coords.y);
    }

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(textures[input.texture_id], sampler_, input.tex_coords);
}
