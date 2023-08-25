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
    textures_count: u32,
    output_resolution: vec2<u32>,
}

struct TextureLayout {
    top: i32,
    left: i32,
    rotation: i32,
    padding: i32, // has to be alligned to 16
}

var<push_constant> common_params: CommonShaderParameters;

@group(0) @binding(0) var textures: binding_array<texture_2d<f32>, 16>;
@group(1) @binding(0) var<uniform> layouts: array<TextureLayout, 16>;
@group(2) @binding(0) var sampler_: sampler;

// Lineary interpolates x from [x_min, x_max] to [y_min, y_max] domain
fn lerp(x: f32, x_min: f32, x_max: f32, y_min: f32, y_max: f32) -> f32 {
    return (x - x_min) / (x_max - x_min) * (y_max - y_min) + y_min;
}

fn rotation_matrix(degrees: i32) -> mat3x3<f32> {
    let pi: f32 = 3.14159;
    let radians: f32 = f32(degrees) * pi / 180.0;
    let col1: vec3<f32> = vec3<f32>(cos(radians), sin(radians), 0.0);
    let col2: vec3<f32> = vec3<f32>(-sin(radians), cos(radians), 0.0);
    let col3: vec3<f32> = vec3<f32>(0.0, 0.0, 1.0);
    return mat3x3<f32>(col1, col2, col3);
}

fn scale_matrix(x_scale: f32, y_scale: f32) -> mat3x3<f32> {
    let col1: vec3<f32> = vec3<f32>(x_scale, 0.0, 0.0);
    let col2: vec3<f32> = vec3<f32>(0.0, y_scale, 0.0);
    let col3: vec3<f32> = vec3<f32>(0.0, 0.0, 1.0);
    return mat3x3<f32>(col1, col2, col3);
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let input_texture_size: vec2<u32> = textureDimensions(textures[input.texture_id]);
    let texture_layout: TextureLayout = layouts[input.texture_id];
    let output_width: f32 = f32(common_params.output_resolution.x);
    let output_height: f32 = f32(common_params.output_resolution.y);

    // pixel coords - ([-output_width / 2, output_width / 2], [-output_height / 2, output_height / 2])
    let pixels_position: vec3<f32> = vec3<f32>(input.position.x, input.position.y, 1.0) *
        vec3<f32>(output_width / 2.0, output_height / 2.0, 1.0);

    // TODO: cacluate that on CPU and send just matrix in uniform
    let x_scale: f32 = f32(input_texture_size.x) / output_width;
    let y_scale: f32 = f32(input_texture_size.y) / output_height;

    let translation: vec3<f32> = vec3<f32>(
        -(output_width / 4.0) + f32(texture_layout.left), 
        (output_height / 4.0) - f32(texture_layout.top),
        0.0
    );
    let rotation_matrix: mat3x3<f32> = rotation_matrix(texture_layout.rotation);
    let scale_matrix: mat3x3<f32> = scale_matrix(x_scale, y_scale);

    let output_position_pixels: vec3<f32> = 
       (pixels_position * rotation_matrix * scale_matrix) + translation;

    output.position = vec4<f32>(output_position_pixels.x * 2.0 / output_width, output_position_pixels.y * 2.0 / output_height, 0.0, 1.0);
    output.tex_coords = input.tex_coords;
    output.texture_id = input.texture_id;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(textures[input.texture_id], sampler_, input.tex_coords);
}
