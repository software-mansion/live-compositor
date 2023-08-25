struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) video_id: i32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) @interpolate(flat) video_id: i32,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    output.position = vec4(input.position, 1.0);
    output.tex_coords = input.tex_coords;

    return output;
}

@group(0) @binding(0) var y_texture: texture_2d<f32>;
@group(0) @binding(1) var u_texture: texture_2d<f32>;
@group(0) @binding(2) var v_texture: texture_2d<f32>;

@group(1) @binding(0) var sampler_: sampler;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let y = textureSample(y_texture, sampler_, input.tex_coords).x;
    let u = textureSample(u_texture, sampler_, input.tex_coords).x;
    let v = textureSample(v_texture, sampler_, input.tex_coords).x;

    let r = y + 1.40200 * (v - 128.0 / 255.0);
    let g = y - 0.34414 * (u - 128.0 / 255.0) - 0.71414 * (v - 128.0 / 255.0);
    let b = y + 1.77200 * (u - 128.0 / 255.0);

    return vec4(clamp(r, 0.0, 1.0), clamp(g, 0.0, 1.0), clamp(b, 0.0, 1.0), 1.0);
}
