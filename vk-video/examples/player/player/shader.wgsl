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

@group(0) @binding(0) var y_texture: texture_2d<f32>;
@group(0) @binding(1) var uv_texture: texture_2d<f32>;
@group(0) @binding(2) var sampler_: sampler;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    var y = textureSample(y_texture, sampler_, input.tex_coords).x;
    var uv = textureSample(uv_texture, sampler_, input.tex_coords);
    var u = uv.x;
    var v = uv.y;

    // https://en.wikipedia.org/wiki/YCbCr#ITU-R_BT.601_conversion
    let r = 1.1643828125 * y + 1.59602734375 * v - 0.87078515625;
    let g = 1.1643828125 * y - 0.39176171875 * u - 0.81296875000 * v + 0.52959375;
    let b = 1.1643828125 * y + 2.01723437500 * u - 1.08139062500;

    return vec4<f32>(clamp(r, 0.0, 1.0), clamp(g, 0.0, 1.0), clamp(b, 0.0, 1.0), 1.0);
}
