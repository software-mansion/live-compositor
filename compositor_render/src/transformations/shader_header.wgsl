struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) texture_id: i32,
}

struct CommonShaderParameters {
    time: f32,
    textures_count: u32,
    output_resolution: vec2<u32>,
}

@group(0) @binding(0) var textures: binding_array<texture_2d<f32>, 16>;
@group(2) @binding(0) var sampler_: sampler;

var<push_constant> common_params: CommonShaderParameters;
