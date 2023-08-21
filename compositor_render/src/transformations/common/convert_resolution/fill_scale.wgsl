struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

struct CommonShaderParameters {
    time: f32,
    textures_count: u32,
    output_texture_size: vec2<u32>,
}

var<push_constant> common_params: CommonShaderParameters;

@group(0) @binding(0) var textures: binding_array<texture_2d<f32>, 16>;
@group(2) @binding(0) var sampler_: sampler;

fn get_scale_matrix(x_scale: f32, y_scale: f32) -> mat4x4<f32> {
    let col1: vec4<f32> = vec4<f32>(x_scale, 0.0, 0.0, 0.0); 
    let col2: vec4<f32> = vec4<f32>(0.0, y_scale, 0.0, 0.0); 
    let col3: vec4<f32> = vec4<f32>(0.0, 0.0, 1.0, 0.0); 
    let col4: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 1.0); 
    return mat4x4<f32>(col1, col2, col3, col4);
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let input_texture_size: vec2<u32> = textureDimensions(textures[0]);
    let input_texture_width: f32 = f32(input_texture_size.x);
    let input_texture_height: f32 = f32(input_texture_size.y);
    let input_ratio: f32 = input_texture_width / input_texture_height;

    let output_texture_width: f32 = f32(common_params.output_texture_size.x);
    let output_texture_height: f32 = f32(common_params.output_texture_size.y);
    let output_ratio: f32 = output_texture_width / output_texture_height;

    var x_scale: f32 = 1.0;
    var y_scale: f32 = 1.0;

    // This transformation preserves the input texture ratio.
    //
    // If the input ratio is larger than the output ratio, the texture is scaled,
    // such that input width = output width. Then:
    // scale_factor_pixels = output_width / input_width
    // Using clip space coords ([-1, 1] range in both axis):
    // scale_factor_x_clip_space = 1.0 (input x coords are already fitted)
    // scale_factor_y_clip_space = scale_factor_pixels * input_width / output_width
    // scale_factor_y_clip_space = (output_height * input_width) / (output_width * input_height)
    // scale_factor_y_clip_space = input_ratio / output_ratio
    //
    // If the output ratio is larger, then the texture is scaled up,
    // such that input_height = output_height.
    // Analogusly:
    // scale_factor_x_clip_space = input_ratio / output_ratio
    // scale_factor_y_clip_space = 1.0 (input y coords are already fitted)
    if input_ratio >= output_ratio {
        y_scale = output_ratio / input_ratio;
    } else {
        x_scale = input_ratio / output_ratio;
    }

    let scale_matrix: mat4x4<f32> = get_scale_matrix(x_scale, y_scale);

    output.position = vec4(input.position, 1.0) * scale_matrix;
    output.tex_coords = input.tex_coords;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(textures[0], sampler_, input.tex_coords);
}
