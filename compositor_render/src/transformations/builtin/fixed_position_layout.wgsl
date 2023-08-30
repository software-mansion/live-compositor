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
    _padding: i32, // has to be alligned to 16
}

var<push_constant> common_params: CommonShaderParameters;

@group(0) @binding(0) var textures: binding_array<texture_2d<f32>, 16>;
@group(1) @binding(0) var<uniform> layouts: array<TextureLayout, 16>;
@group(2) @binding(0) var sampler_: sampler;

fn rotation_matrix(degrees: i32) -> mat3x3<f32> {
    let radians: f32 = radians(f32(degrees));
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
    if input.texture_id == -1 {
        output.position = vec4<f32>(input.position, 1.0);
        output.tex_coords = input.tex_coords;
        output.texture_id = 0;
        return output;
    }

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
    let sample = textureSample(textures[input.texture_id], sampler_, input.tex_coords);
    
    if common_params.textures_count == 0u {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    return sample;
}
