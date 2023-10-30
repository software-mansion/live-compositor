struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) layout_id: i32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) @interpolate(flat) layout_id: i32,
}


struct Layout {
    position_transformation: mat4x4<f32>,
    texture_id: i32,
    background_color: vec4<f32>,
}

@group(0) @binding(0) var textures: binding_array<texture_2d<f32>, 16>;
@group(1) @binding(0) var<uniform> layouts: array<Layout, 128>;
@group(2) @binding(0) var sampler_: sampler;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    
    // 0 inputs case (TODO: not sure if this can happen)
    if input.layout_id == -1 {
        output.position = vec4<f32>(input.position, 1.0);
        output.tex_coords = input.tex_coords;
        output.layout_id = 0;
        return output;
    }
    
    let transformation_matrix: mat4x4<f32> = layouts[input.layout_id].position_transformation;

    output.position = vec4(input.position, 1.0) * transformation_matrix;
    output.tex_coords = input.tex_coords;
    output.layout_id = input.layout_id;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let current_layout = layouts[input.layout_id];

    // sampling can't be conditional, so in case of texture_id == -1
    // sample textures[0], but ignore the result.
    var texture_id = current_layout.texture_id;
    if texture_id == -1 {
        texture_id = 0;
    }
    let sample = textureSample(textures[texture_id], sampler_, input.tex_coords);

    if current_layout.texture_id != -1 {
        return sample;
    } else {
        return current_layout.background_color;
    }
}
