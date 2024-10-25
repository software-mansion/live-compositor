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
    // Position relative to center of the rectangle in [-rect_width/2, rect_width/2] X [-rect_height/2, height/2]
    @location(2) center_position: vec2<f32>
}

struct BoxShadowParams {
    border_radius: vec4<f32>,
    color: vec4<f32>,
    top: f32,
    left: f32,
    width: f32,
    height: f32,
    rotation_degrees: f32,
    blur_radius: f32,
}

struct TextureParams {
    border_radius: vec4<f32>,
    border_color: vec4<f32>,
    // position
    top: f32,
    left: f32,
    width: f32,
    height: f32,
    // texture crop
    crop_top: f32,
    crop_left: f32,
    crop_width: f32,
    crop_height: f32,

    rotation_degrees: f32,
    // border size in pixels
    border_width: f32,
}

struct ColorParams {
    border_radius: vec4<f32>,
    border_color: vec4<f32>,
    color: vec4<f32>,

    top: f32,
    left: f32,
    width: f32,
    height: f32,

    rotation_degrees: f32,
    border_width: f32,
}

struct ParentBorderRadius {
    radius: vec4<f32>,
    top: f32,
    left: f32,
    width: f32,
    height: f32,
}

struct LayoutInfo {
    // 0 -> Texture, 1 -> Color, 2 -> BoxShadow
    layout_type: u32,
    index: u32,
    parent_masks_len: u32
}


@group(0) @binding(0) var texture: texture_2d<f32>;

@group(1) @binding(0) var<uniform> output_resolution: vec4<f32>;
@group(1) @binding(1) var<uniform> texture_params: array<TextureParams, 100>;
@group(1) @binding(2) var<uniform> color_params: array<ColorParams, 100>;
@group(1) @binding(3) var<uniform> box_shadow_params: array<BoxShadowParams, 100>;

@group(2) @binding(0) var<uniform> parent_masks: array<ParentBorderRadius, 20>;

@group(3) @binding(0) var sampler_: sampler;

var<push_constant> layout_info: LayoutInfo;

fn rotation_matrix(rotation: f32) -> mat4x4<f32> {
    // wgsl is column-major
    let angle = radians(rotation);
    let c = cos(angle);
    let s = sin(angle);
    return mat4x4<f32>(
        vec4<f32>(c, s, 0.0, 0.0),
        vec4<f32>(-s, c, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 1.0, 0.0),
        vec4<f32>(0.0, 0.0, 0.0, 1.0)
    );
}

fn scale_matrix(scale: vec2<f32>) -> mat4x4<f32> {
    return mat4x4<f32>(
        vec4<f32>(scale.x, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, scale.y, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 1.0, 0.0),
        vec4<f32>(0.0, 0.0, 0.0, 1.0)
    );
}


fn translation_matrix(translation: vec2<f32>) -> mat4x4<f32> {
    return mat4x4<f32>(
        vec4<f32>(1.0, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, 1.0, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 1.0, 0.0),
        vec4<f32>(translation, 0.0, 1.0)
    );
}

fn vertices_transformation_matrix(left: f32, top: f32, width: f32, height: f32, rotation: f32) -> mat4x4<f32> {    
    let scale_to_size = vec2<f32>(
        width / output_resolution.x,
        height / output_resolution.y
    );
    let scale_to_pixels = vec2<f32>(
        output_resolution.x / 2.0,
        output_resolution.y / 2.0
    );
    let scale_to_clip_space = vec2<f32>(
        1.0 / scale_to_pixels.x,
        1.0 / scale_to_pixels.y
    );

    let scale_to_pixels_mat = scale_matrix(scale_to_pixels * scale_to_size);
    let scale_to_clip_space_mat = scale_matrix(scale_to_clip_space);

    let left_border_x = -(output_resolution.x / 2.0);
    let distance_left_to_middle = left + width / 2.0;
    let top_border_y = output_resolution.y / 2.0;
    let distance_top_to_middle = top + height / 2.0;
    let translation = vec2<f32>(
        left_border_x + distance_left_to_middle,
        top_border_y - distance_top_to_middle
    );

    let translation_mat = translation_matrix(translation);
    let rotation_mat = rotation_matrix(rotation);

    return scale_to_clip_space_mat * translation_mat * rotation_mat * scale_to_pixels_mat;
}

fn texture_coord_transformation_matrix(crop_left: f32, crop_top: f32, crop_width: f32, crop_height: f32) -> mat4x4<f32> {
    let dim = textureDimensions(texture);
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

    switch (layout_info.layout_type) {
        // texture
        case 0u: {
            let vertices_transformation = vertices_transformation_matrix(
                texture_params[layout_info.index].left,
                texture_params[layout_info.index].top,
                texture_params[layout_info.index].width,
                texture_params[layout_info.index].height,
                texture_params[layout_info.index].rotation_degrees
            );
            let texture_transformation = texture_coord_transformation_matrix(
                texture_params[layout_info.index].crop_left,
                texture_params[layout_info.index].crop_top,
                texture_params[layout_info.index].crop_width,
                texture_params[layout_info.index].crop_height
            );
            output.position = vertices_transformation * vec4(input.position, 1.0);
            output.tex_coords = (texture_transformation * vec4<f32>(input.tex_coords, 0.0, 1.0)).xy;
            let rect_size = vec2<f32>(texture_params[layout_info.index].width, texture_params[layout_info.index].height);
            output.center_position = input.position.xy / 2.0 * rect_size;
        }
        // color
        case 1u: {
            let vertices_transformation = vertices_transformation_matrix(
                color_params[layout_info.index].left,
                color_params[layout_info.index].top,
                color_params[layout_info.index].width,
                color_params[layout_info.index].height,
                color_params[layout_info.index].rotation_degrees
            );
            output.position = vertices_transformation * vec4(input.position, 1.0);
            output.tex_coords = input.tex_coords;
            let rect_size = vec2<f32>(color_params[layout_info.index].width, color_params[layout_info.index].height);
            output.center_position = input.position.xy / 2.0 * rect_size;
        }
        // box shadow
        case 2u:  {
            let width = box_shadow_params[layout_info.index].width + 2.0 * box_shadow_params[layout_info.index].blur_radius;
            let height = box_shadow_params[layout_info.index].height + 2.0 * box_shadow_params[layout_info.index].blur_radius;

            let vertices_transformation = vertices_transformation_matrix(
                box_shadow_params[layout_info.index].left - box_shadow_params[layout_info.index].blur_radius,
                box_shadow_params[layout_info.index].top - box_shadow_params[layout_info.index].blur_radius,
                width,
                height,
                box_shadow_params[layout_info.index].rotation_degrees
            );
            output.position = vertices_transformation * vec4(input.position, 1.0);
            output.tex_coords = input.tex_coords;
            let rect_size = vec2<f32>(width, height);
            output.center_position = input.position.xy / 2.0 * rect_size;
        }
        default {}
    }

    return output;
}

// Signed distance function for rounded rectangle https://iquilezles.org/articles/distfunctions
// adapted from https://www.shadertoy.com/view/4llXD7
// dist - signed distance from the center of the rectangle in pixels
// size - size of the rectangle in pixels
// radius - radius of the corners in pixels [top-left, top-right, bottom-right, bottom-left]
// rotation - rotation of the rectangle in degrees
// WARNING - it doesn't work when border radius is > min(size.x, size.y) / 2
fn roundedRectSDF(dist: vec2<f32>, size: vec2<f32>, radius: vec4<f32>, rotation: f32) -> f32 {
    let half_size = size / 2.0;
    
    // wierd hack to get the radius of the nearest corner stored in r.x
    var r: vec2<f32> = vec2<f32>(0.0, 0.0);
    r = select(radius.yz, radius.xw, dist.x < 0.0 );
    r.x = select(r.y, r.x, dist.y < 0.0 );

    let q = abs(dist) - half_size + r.x;
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0, 0.0))) - r.x;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let transparent = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    // TODO: Add parent mask handling
    let parent_mask_alpha = 1.0;

    switch layout_info.layout_type {
        case 0u: {
            let sample = textureSample(texture, sampler_, input.tex_coords);

            let width = texture_params[layout_info.index].width;
            let height = texture_params[layout_info.index].height;
            let border_radius = texture_params[layout_info.index].border_radius;
            let rotation_degrees = texture_params[layout_info.index].rotation_degrees;
            let border_width = texture_params[layout_info.index].border_width;
            let border_color = texture_params[layout_info.index].border_color;

            let size = vec2<f32>(width, height);
            let edge_distance = roundedRectSDF(
                input.center_position,
                size, 
                border_radius, 
                rotation_degrees
            );

            let smoothed_alpha = 1.0 - smoothstep(0.0, 2.0, edge_distance);
            let border_alpha = 1.0 - smoothstep(-border_width + 1.0, -border_width, edge_distance);

            let mixed_background = mix(transparent, sample, min(smoothed_alpha, parent_mask_alpha));
            let mixed_border = mix(mixed_background, border_color, min(border_alpha, smoothed_alpha));
            return mixed_border;
        }
        case 1u: {
            let color = color_params[layout_info.index].color;

            let width = color_params[layout_info.index].width;
            let height = color_params[layout_info.index].height;
            let border_radius = color_params[layout_info.index].border_radius;
            let rotation_degrees = color_params[layout_info.index].rotation_degrees;
            let border_width = color_params[layout_info.index].border_width;
            let border_color = color_params[layout_info.index].border_color;

            let size = vec2<f32>(width, height);
            let edge_distance = roundedRectSDF(
                input.center_position,
                size, 
                border_radius, 
                rotation_degrees
            );

            let smoothed_alpha = 1.0 - smoothstep(0.0, 2.0, edge_distance);
            let border_alpha = 1.0 - smoothstep(-border_width + 1.0, -border_width, edge_distance);

            let mixed_background = mix(transparent, color, min(smoothed_alpha, parent_mask_alpha));
            let mixed_border = mix(mixed_background, border_color, border_alpha);
            return mixed_border;
        }
        case 2u: {
            let color = box_shadow_params[layout_info.index].color;

            let width = box_shadow_params[layout_info.index].width;
            let height = box_shadow_params[layout_info.index].height;
            let border_radius = box_shadow_params[layout_info.index].border_radius;
            let rotation_degrees = box_shadow_params[layout_info.index].rotation_degrees;
            let blur_radius = box_shadow_params[layout_info.index].blur_radius;

            let size = vec2<f32>(width, height);
            let edge_distance = roundedRectSDF(
                input.center_position,
                size, 
                border_radius, 
                rotation_degrees
            );

            let smoothed_alpha = 1.0 - smoothstep(0.0, blur_radius, edge_distance);
            let mixed_background = mix(transparent, color, min(smoothed_alpha, parent_mask_alpha));
            return mixed_background;
        }
        default {
            return vec4(0.0, 0.0, 0.0, 0.0);
        }
    }
}
