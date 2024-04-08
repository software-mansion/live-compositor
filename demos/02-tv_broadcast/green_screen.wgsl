struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    output.position = vec4(input.position, 1.0);
    output.tex_coords = input.tex_coords;

    return output;
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

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let greenScreenTexture = textureSample(textures[0], sampler_, input.tex_coords);

    if (greenScreenTexture.g > 0.5 && greenScreenTexture.r < 0.5 && greenScreenTexture.b < 0.5) {
        return vec4(0.0, 0.0, 0.0, 0.0);
    } else {
        return vec4(greenScreenTexture.r, greenScreenTexture.g, greenScreenTexture.b, greenScreenTexture.a);
    }
}
