// 0. News
// 1. TV
// 2. Background
// 3. Text
// 4. Bunny

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
    if (plane_id == 4) {
        return bunny_translation_matrix * bunny_scale_down_matrix * vec4(input.position, 1.0);
    } else {
        return vec4(input.position, 1.0);
    }
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    output.position = default_position(input, base_params.plane_id);
    output.tex_coords = input.tex_coords;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let sample: vec4<f32> = textureSample(textures[base_params.plane_id], sampler_, input.tex_coords);

    if (base_params.plane_id == 0) {
        if (base_params.time < 19.0) {
            return vec4(0.0, 0.0, 0.0, 0.0);
        } else if (base_params.time < 20.0) {
            return mix(
                vec4(0.0, 0.0, 0.0, 0.0),
                sample,
                (base_params.time - 19.0)
            );
        } else {
            return sample;
        }
    } else if (base_params.plane_id == 1) {
        return mix(
            sample,
            vec4(0.0, 0.0, 0.0, 0.0),
            smoothstep(0.6, 0.9, dot(normalize(sample.rgb), normalize(vec3(0.0, 1.0, 0.0))))
        );
    } else if (base_params.plane_id == 2) {
        if (base_params.time < 31.0) {
            return vec4(0.0, 0.0, 0.0, 0.0);
        } else if (base_params.time < 32.0) {
            return mix(
                vec4(0.0, 0.0, 0.0, 0.0),
                sample,
                (base_params.time - 31.0)
            );
        } else {
            return sample;
        }
    } else if (base_params.plane_id == 3) {
        if (base_params.time < 38.0) {
            return vec4(0.0, 0.0, 0.0, 0.0);
        } else if (base_params.time < 39.0) {
            return mix(
                vec4(0.0, 0.0, 0.0, 0.0),
                sample,
                (base_params.time - 38.0)
            );
        } else {
            return sample;
        }
    } else {
        if (base_params.time < 25.0) {
            return vec4(0.0, 0.0, 0.0, 0.0);
        } else if (base_params.time < 26.0) {
            return mix(
                vec4(0.0, 0.0, 0.0, 0.0),
                sample,
                (base_params.time - 25.0)
            );
        } else {
            return sample;
        }
    }

    return sample;
}
