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
    vertices_transformation: mat4x4<f32>,
    texture_coord_transformation: mat4x4<f32>,
    color: vec4<f32>, // used only when texture_id = -1
    texture_id: i32,
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
    
    let vertices_transformation_matrix: mat4x4<f32> = layouts[input.layout_id].vertices_transformation;
    let texture_coord_transformation_matrix: mat4x4<f32> = layouts[input.layout_id].texture_coord_transformation;

    output.position = vec4(input.position, 1.0) * vertices_transformation_matrix;
    output.tex_coords = (vec4(input.tex_coords, 0.0, 1.0) * texture_coord_transformation_matrix).xy;
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
    // clamp transparent, when crop > input texture
    let is_inside: f32 = round(f32(input.tex_coords.x < 1.0 && input.tex_coords.x > 0.0 && input.tex_coords.y > 0.0 && input.tex_coords.y < 1.0));
    
    let sample = is_inside * textureSample(textures[texture_id], sampler_, input.tex_coords) 
        + (1.0 - is_inside) * vec4<f32>(0.0, 0.0, 0.0, 0.0);

    if current_layout.texture_id != -1 {
        return sample;
    } else {
        return current_layout.color;
    }
}
