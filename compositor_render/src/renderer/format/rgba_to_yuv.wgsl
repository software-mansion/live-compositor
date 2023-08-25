struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) input_id: i32,
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

@group(0) @binding(0) var texture: texture_2d<f32>;
@group(1) @binding(0) var sampler_: sampler;
@group(2) @binding(0) var<uniform> plane_selector: u32;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) f32 {
    let color = textureSample(texture, sampler_, input.tex_coords);
    var conversion_weights: vec4<f32>;
    var conversion_bias: f32;

    if(plane_selector == 0u) {
        // Y
        conversion_weights = vec4<f32>(0.299, 0.587, 0.114, 0.0);
        conversion_bias = 0.0;
    } else if(plane_selector == 1u) {
        // U
        conversion_weights = vec4<f32>(-0.168736, -0.331264, 0.5, 0.0);
        conversion_bias = 128.0 / 255.0;
    } else if(plane_selector == 2u) {
        // V
        conversion_weights = vec4<f32>(0.5, -0.418688, -0.081312, 0.0);
        conversion_bias = 128.0 / 255.0;
    } else {
        conversion_weights = vec4<f32>();
    }

    return clamp(dot(color, conversion_weights) + conversion_bias, 0.0, 1.0);
}
