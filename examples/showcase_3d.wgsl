struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

struct BaseShaderParameters {
    plane_id: i32,
    time: f32,
    output_resolution: vec2<u32>,
    texture_count: u32,
}

@group(0) @binding(0) var textures: binding_array<texture_2d<f32>, 16>;
@group(2) @binding(0) var sampler_: sampler;

var<push_constant> base_params: BaseShaderParameters;

const sin_60 = 0.866;
const neg_sin_60 = -0.866;
const cos_60 = 0.5;
const neg_cos_60 = -0.5;

const sin_30 = 0.5;
const neg_sin_30 = -0.5;
const cos_30 = 0.866;
const neg_cos_30 = -0.866;

const sin_20 = 0.342;
const neg_sin_20 = -0.342;
const cos_20 = 0.94;
const neg_cos_20 = -0.94;


const scale_down_matrix: mat4x4<f32> = mat4x4(
    vec4(0.42, 0.0, 0.0, 0.0),
    vec4(0.0, 0.47, 0.0, 0.0),
    vec4(0.0, 0.0, 1.0, 0.0),
    vec4(0.0, 0.0, 0.0, 1.0)
);

const rotation_matrix_x: mat4x4<f32> = mat4x4(
    vec4(1.0, 0.0, 0.0, 0.0),
    vec4(0.0, cos_30, sin_30, 0.0),
    vec4(0.0, neg_sin_30, cos_30, 0.0),
    vec4(0.0, 0.0, 0.0, 1.0)

);

const rotation_matrix_y: mat4x4<f32> = mat4x4(
    vec4(cos_30, 0.0, neg_sin_30, 0.0),
    vec4(0.0, 1.0, 0.0, 0.0),
    vec4(sin_30, 0.0, cos_30, 0.0),
    vec4(0.0, 0.0, 0.0, 1.0)
);

fn in_animation(input: VertexInput, time: f32, plane_id: i32) -> VertexOutput {
    if (plane_id == 0) {
        return VertexOutput(vec4(input.position.xy, 0.0, 1.0), input.tex_coords);
    }

    var x_translation: f32 = 0.0;
    var y_translation: f32 = 0.0;
    var z_translation: f32 = 0.0;

    // TV background
    if (plane_id == 1) {
        x_translation = -0.20;
        y_translation = 0.40;
        z_translation = 0.1 + 0.35;
    }
    // TV
    else if (plane_id == 2) {
        x_translation = 0.0;
        y_translation = 0.0;
        z_translation = 0.2 + 0.35;
    }
    // Bunny
    else if (plane_id == 3) {
        x_translation = 0.20;
        y_translation = -0.40;
        z_translation = 0.2 + 0.35;
    };

    let translation_matrix: mat4x4<f32> = mat4x4(
        vec4(1.0, 0.0, 0.0, 0.0),
        vec4(0.0, 1.0, 0.0, 0.0),
        vec4(0.0, 0.0, 1.0, 0.0),
        vec4(x_translation, y_translation, z_translation, 1.0)
    );

    let start_position = default_position(input, plane_id);
    let in = vec4(input.position, 1.0);
    let transformed_input: vec4<f32> = translation_matrix * rotation_matrix_x * rotation_matrix_y * scale_down_matrix * in;

    if (time < 4.0) {
        return VertexOutput(
            lerp(start_position, transformed_input, (time - 3.0) / 1.0),
            input.tex_coords
        );
    } else if (time < 7.0) {
        return VertexOutput(
            transformed_input,
            input.tex_coords
        );
    } else {
        return VertexOutput(
            lerp(transformed_input, start_position, (time - 7.0) / 1.0),
            input.tex_coords
        );
    }
}

fn lerp(a: vec4<f32>, b: vec4<f32>, t: f32) -> vec4<f32> {
    return a + (b - a) * t;
}

const bunny_scale_down_matrix: mat4x4<f32> = mat4x4(
    vec4(0.33, 0.0, 0.0, 0.0),
    vec4(0.0, 0.33, 0.0, 0.0),
    vec4(0.0, 0.0, 1.0, 0.0),
    vec4(0.0, 0.0, 0.0, 1.0)
);

const bunny_translation_matrix: mat4x4<f32> = mat4x4(
    vec4(1.0, 0.0, 0.0, 0.0),
    vec4(0.0, 1.0, 0.0, 0.0),
    vec4(0.0, 0.0, 1.0, 0.0),
    vec4(-0.65, 0.62, 0.0, 1.0)
);

fn default_position(input: VertexInput, plane_id: i32) -> vec4<f32> {
    if (plane_id == 3) {
        return bunny_translation_matrix * bunny_scale_down_matrix * vec4(input.position, 1.0);
    } else {
        return vec4(input.position, 1.0);
    }
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    if (base_params.time < 3.0) {
        output.position =  default_position(input, base_params.plane_id);
        output.tex_coords = input.tex_coords;
    } else if (base_params.time < 8.0) {
        return in_animation(input, base_params.time, base_params.plane_id);
    } else {
        output.position =  default_position(input, base_params.plane_id);
        output.tex_coords = input.tex_coords;
    }

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let sample: vec4<f32> = textureSample(textures[base_params.plane_id], sampler_, input.tex_coords);

    // TV
    if (base_params.plane_id == 2) {
        var threshold: f32 = 0.11;
        if (base_params.time < 3.0) {
            threshold = 0.11;
        } else if (base_params.time < 4.0) {
            threshold = 0.11 * (4.0 - base_params.time);
        } else if (base_params.time < 7.0) {
            threshold = 0.0;
        } else if (base_params.time < 8.0) {
            threshold = (base_params.time - 7.0) * 0.11;
        } else if (base_params.time < 9.0) {
            threshold = 0.11;
        } else if (base_params.time < 10.0) {
            threshold = 0.11 * (1.0 - (base_params.time - 9.0));
        } else {
            threshold = 0.0;
        }

        if (input.tex_coords.y > (1.0 - threshold)) {
            return vec4(0.0, 0.0, 0.0, 0.0);
        }

        var alpha: f32 = 0.0;
        if (base_params.time < 3.0) {
            alpha = 0.0;
        } else if (base_params.time < 4.0) {
            alpha = (base_params.time - 3.0);
        } else if (base_params.time < 7.0) {
            alpha = 1.0;
        } else if (base_params.time < 8.0) {
            alpha = 1.0 - (base_params.time - 7.0);
        } else if (base_params.time < 9.0) {
            alpha = 0.0;
        } else if (base_params.time < 10.0) {
            alpha = (base_params.time - 9.0);
        } else if (base_params.time < 13.0) {
            alpha = 1.0;
        } else if (base_params.time < 14.0) {
            alpha = 1.0 - (base_params.time - 13.0);
        } else {
            alpha = 0.0;
        }

        return mix(
            sample,
            vec4(0.0, alpha, 0.0, alpha),
            smoothstep(0.6, 0.9, dot(normalize(sample.rgb), normalize(vec3(0.0, 1.0, 0.0))))
        );
    } else {
        if (base_params.time < 9.0) {
            return sample;
        } else if (base_params.time < 10.0) {
            return mix(
                sample,
                vec4(0.0, 0.0, 0.0, 0.0),
                (base_params.time - 9.0)
            );
        } else {
            return vec4(0.0, 0.0, 0.0, 0.0);
        }

        return sample;
    }   
}
