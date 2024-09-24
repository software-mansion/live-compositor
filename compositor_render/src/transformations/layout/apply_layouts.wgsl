struct VertexInput {
    // position in clip space [-1, -1] (bottom-left) X [1, 1] (top-right)
    @location(0) position: vec3<f32>,
    // texture coordinates in texture coordiantes [0, 0] (top-left) X [1, 1] (bottom-right)
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    // position in output in pixel coordinates [0, 0] (top-left) X [output_resolution.x, output_resolution.y] (bottom-right)
    @builtin(position) position: vec4<f32>,
    // texture coordinates in texture coordiantes [0, 0] (top-left) X [1, 1] (bottom-right)
    @location(0) tex_coords: vec2<f32>,
}

struct Layout {
    // position of the rectangle in output in pixel coordinates [0, 0] (top-left) X [output_resolution.x, output_resolution.y] (bottom-right)
    top: f32,
    left: f32,
    // size of the rectangle in pixels
    width: f32,
    height: f32,
    // texture crop params in pixels
    crop_top: f32,
    crop_left: f32,
    crop_width: f32,
    crop_height: f32,
    // color of the rectangle / box shadow, only used when render_type == 1 || 2 (color or box shadow)
    color: vec4<f32>,
    // border color, in rgba [0.0-255.0] scale
    border_color: vec4<f32>,
    // radius of corners in pixels [top-left, top-right, bottom-right, bottom-left]
    border_radius: vec4<f32>,
    // rotation in degrees
    rotation: f32,
    // border width in pixels, only used if render_type == 0 || 1 (texture or color)
    border: f32,
    // blur of the box shadow in pixels, only used if render_type == 2 (box shadow)
    blur: f32,
    // render type, 0 -> texture, 1 -> color, 2 -> box shadow
    render_type: u32,
    // accutal length of the parent_rounded_borders array
    parent_rounded_borders_len: u32,
}

struct ParentRoundedCorners {
    border_radius: vec4<f32>,
    // position of the parent in output in pixel coordinates [0, 0] (top-left) X [output_resolution.x, output_resolution.y] (bottom-right)
    top: f32,
    left: f32,
    // size of the parent in pixels
    width: f32,
    height: f32,
    // rotation in degrees
    rotation: f32,
}

@group(0) @binding(0) var texture: texture_2d<f32>;
@group(1) @binding(0) var<uniform> current_layout: Layout;
// TODO try using dynamically sized storage buffers instead
@group(1) @binding(1) var<uniform> parent_rounded_borders: array<ParentRoundedCorners, 100>;

@group(2) @binding(0) var sampler_: sampler;
@group(2) @binding(1) var<uniform> output_resolution: vec2<u32>;

fn rotation_matrix(rotation: f32) -> mat3x3<f32> {
    // wgsl is column-major
    let angle = radians(rotation);
    let c = cos(angle);
    let s = sin(angle);
    return mat3x3<f32>(
        vec3<f32>(c, s, 0.0),
        vec3<f32>(-s, c, 0.0),
        vec3<f32>(0.0, 0.0, 1.0)
    );
}

fn scale_matrix(scale: vec2<f32>) -> mat3x3<f32> {
    return mat3x3<f32>(
        vec3<f32>(scale.x, 0.0, 0.0),
        vec3<f32>(0.0, scale.y, 0.0),
        vec3<f32>(0.0, 0.0, 1.0)
    );
}


fn translation_matrix(translation: vec2<f32>) -> mat3x3<f32> {
    return mat3x3<f32>(
        vec3<f32>(1.0, 0.0, 0.0),
        vec3<f32>(0.0, 1.0, 0.0),
        vec3<f32>(translation, 1.0)
    )
}

fn vertices_transformation_matrix(left: f32, top: f32, width: f32, height: f32) -> mat3x3<f32> {
    let scale = vec2<f32>(
        width / output_resolution.x as f32,
        height / output_resolution.y as f32,
    );

    // center of the rectangle in clip space coordinates
    // center in output pixel coords -> left + width / 2
    // scaling to clip space -> 2 * (left + width / 2) / output_resolution.x - 1
    let center = vec2<f32>(
        2.0 * (left + width / 2.0) / f32(output_resolution.x) - 1.0,
        2.0 * (top + height / 2.0) / f32(output_resolution.y) - 1.0,
    );

    return translation_matrix(center) * rotation_matrix(current_layout.rotate) * scale_matrix(scale);
}

fn texture_coord_transformation_matrix(crop_left: f32, crop_top: f32, crop_width: f32, crop_height: f32) -> mat3x3<f32> {
    let dim = textureDimentsions(texture);
    let scale = vec2<f32>(
        crop_width / f32(dim.x),
        crop_height / f32(dim.y),
    );

    let translation = vec2<f32>(
        crop_left / f32(dim.x),
        crop_top / f32(dim.y),
    );

    return translation_matrix(translation) * scale_matrix(scale);
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    
    if (current_layout.render_type == 2u) {
        // make rectagle bigger due to blur effect
        let left = current_layout.left - current_layout.blur;
        let top = current_layout.top - current_layout.blur;
        let width = current_layout.width + 2.0 * current_layout.blur;
        let height = current_layout.height + 2.0 * current_layout.blur;
        let vertices_transformation = vertices_transformation_matrix(left, top, width, height);
        output.position = vec4<f32>(vertices_transformation * input.position, 1.0);
        output.tex_coords = input.tex_coords;    
    } else {
        let vertices_transformation = vertices_transformation_matrix(
            current_layout.left,
            current_layout.top,
            current_layout.width,
            current_layout.height
        );
        let texture_transformation = texture_coord_transformation_matrix(
            current_layout.crop_left,
            current_layout.crop_top,
            current_layout.crop_width,
            current_layout.crop_height
        );
        output.position = vec4<f32>(vertices_transformation * input.position, 1.0);
        output.tex_coords = texture_transformation *  input.tex_coords
    }

    return output;
}

// Signed distance function for rounded rectangle https://iquilezles.org/articles/distfunctions
// adapted from https://www.shadertoy.com/view/4llXD7
// position - signed distance from the center of the rectangle in pixels
// size - size of the rectangle in pixels
// radius - radius of the corners in pixels [top-left, top-right, bottom-right, bottom-left]
// rotation - rotation of the rectangle in degrees
fn roundedRectSDF(position: vec2<f32>, size: vec2<f32>, radius: vec4<f32>, rotation: f32) -> f32 {
    let half_size = size / 2.0;
    let rotated_position = vec2<f32>(
        cos(radians(rotation)) * position.x - sin(radians(rotation)) * position.y,
        sin(radians(rotation)) * position.x + cos(radians(rotation)) * position.y
    );

    // wierd hack to get the radius of the nearest corner stored in r.x
    var r: vec2<f32> = radius.xy;
    r = select(radius.zw, r, rotated_position.x >= 0 );
    r.x = select(r.y, r.x, rotated_position.y >= 0 );

    let q = abs(rotated_position) - half_size + r.x;
    return min(max(q.x, q.y), 0.0) + length(max(q, 0.0)) - r.x;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let sample = textureSample(texture, sampler_, input.tex_coords);
    let size = vec2<f32>(current_layout.width, current_layout.height);
    let center = vec2<f32>(
        current_layout.left + current_layout.width / 2.0,
        current_layout.top + current_layout.height / 2.0
    );
    let position = input.position.xy - center;
    let edge_sdf = roundedRectSDF(position, size, current_layout.border_radius, current_layout.rotation);
    let border_color_alpha = select(0.0, 1.0, edge_sdf > -current_layout.border);
    let sample_alpha = 1.0 - border_color_alpha;

    switch current_layout.render_type {
        case 0u: {
            // clamp transparent, when crop > input texture
            let is_inside: f32 = round(f32(input.tex_coords.x < 1.0 && input.tex_coords.x > 0.0 && input.tex_coords.y > 0.0 && input.tex_coords.y < 1.0));
            let mixed = mix(current_layout.border_color, sample, border_color_alpha);
            return is_inside * mixed;
        }
        case 1u: {
            let mixed = mix(current_layout.border_color, current_layout.color, border_color_alpha);
            return mixed;
        }
        case 2u: {
            return vec4(1.0, 1.0, 1.0, 1.0);
        }
    }
}
