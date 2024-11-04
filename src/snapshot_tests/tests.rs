use std::time::Duration;

use compositor_render::{
    image::{ImageSource, ImageSpec, ImageType},
    shader::ShaderSpec,
    RendererId, RendererSpec, Resolution,
};
use serde_json::{json, Value};

use super::test_case::{TestCase, TestInput, Updates};

const DEFAULT_RESOLUTION: Resolution = Resolution {
    width: 640,
    height: 360,
};

pub fn snapshot_tests() -> Vec<TestCase> {
    let mut tests = Vec::new();
    tests.append(&mut base_snapshot_tests());
    tests.append(&mut view_snapshot_tests());
    tests.append(&mut transition_snapshot_tests());
    tests.append(&mut image_snapshot_tests());
    tests.append(&mut text_snapshot_tests());
    tests.append(&mut tiles_snapshot_tests());
    tests.append(&mut rescaler_snapshot_tests());
    tests.append(&mut shader_snapshot_tests());
    tests
}

fn shader_snapshot_tests() -> Vec<TestCase> {
    let mut base_params_snapshot_tests = shader_base_params_snapshot_tests();
    let mut user_params_snapshot_tests = shader_user_params_snapshot_tests();

    base_params_snapshot_tests.append(&mut user_params_snapshot_tests);
    base_params_snapshot_tests
}

fn shader_user_params_snapshot_tests() -> Vec<TestCase> {
    struct CircleLayout {
        pub left_px: u32,
        pub top_px: u32,
        pub width_px: u32,
        pub height_px: u32,
        /// RGBA 0.0 - 1.0 range
        pub background_color: [f32; 4],
    }

    impl CircleLayout {
        pub fn shader_param(&self) -> Value {
            let background_color_params: Vec<Value> = self
                .background_color
                .iter()
                .map(|val| {
                    {
                        json!({
                            "type": "f32",
                            "value": val
                        })
                    }
                })
                .collect();

            json!({
                "type": "struct",
                "value": [
                    {
                        "field_name": "left_px",
                        "type": "u32",
                        "value": self.left_px
                    },
                    {
                        "field_name": "top_px",
                        "type": "u32",
                        "value": self.top_px
                    },
                    {
                        "field_name": "width_px",
                        "type": "u32",
                        "value": self.width_px
                    },
                    {
                        "field_name": "height_px",
                        "type": "u32",
                        "value": self.height_px
                    },
                    {
                        "field_name": "background_color",
                        "type": "list",
                        "value": background_color_params
                    },
                ]
            })
        }
    }

    let input1 = TestInput::new(1);
    let input2 = TestInput::new(2);
    let input3 = TestInput::new(3);
    let input4 = TestInput::new(4);

    const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
    const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
    const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
    const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

    let circle_layout_shader = (
        RendererId("user_params_circle_layout".into()),
        RendererSpec::Shader(ShaderSpec {
            source: include_str!("../../snapshot_tests/shader/circle_layout.wgsl").into(),
        }),
    );

    let layout1 = CircleLayout {
        left_px: 0,
        top_px: 0,
        width_px: (DEFAULT_RESOLUTION.width / 2) as u32,
        height_px: (DEFAULT_RESOLUTION.height / 2) as u32,
        background_color: RED,
    };

    let layout2 = CircleLayout {
        left_px: (DEFAULT_RESOLUTION.width / 2) as u32,
        top_px: 0,
        width_px: (DEFAULT_RESOLUTION.width / 2) as u32,
        height_px: (DEFAULT_RESOLUTION.height / 2) as u32,
        background_color: GREEN,
    };

    let layout3 = CircleLayout {
        left_px: 0,
        top_px: (DEFAULT_RESOLUTION.height / 2) as u32,
        width_px: (DEFAULT_RESOLUTION.width / 2) as u32,
        height_px: (DEFAULT_RESOLUTION.height / 2) as u32,
        background_color: BLUE,
    };

    let layout4 = CircleLayout {
        left_px: (DEFAULT_RESOLUTION.width / 2) as u32,
        top_px: (DEFAULT_RESOLUTION.height / 2) as u32,
        width_px: (DEFAULT_RESOLUTION.width / 2) as u32,
        height_px: (DEFAULT_RESOLUTION.height / 2) as u32,
        background_color: WHITE,
    };

    let shader_param = json!({
            "type": "list",
            "value": [
                layout1.shader_param(),
                layout2.shader_param(),
                layout3.shader_param(),
                layout4.shader_param(),
            ]
    });

    let inputs = Vec::from([input1, input2, input3, input4]);

    let children: Vec<Value> = inputs
        .iter()
        .map(|input| {
            json!({
                "type": "input_stream",
                "input_id": input.name
            })
        })
        .collect();

    let circle_layout_scene = Box::new(
        json!({
            "video": {
                "root": {
                    "type": "shader",
                    "shader_id": "user_params_circle_layout",
                    "resolution": {
                        "width": DEFAULT_RESOLUTION.width,
                        "height": DEFAULT_RESOLUTION.height
                    },
                    "shader_param": shader_param,
                    "children": children,
                }
            }
        })
        .to_string(),
    );

    Vec::from([TestCase {
        name: "shader/user_params_circle_layout",
        scene_updates: Updates::Scene(circle_layout_scene.leak(), DEFAULT_RESOLUTION),
        renderers: vec![circle_layout_shader],
        inputs,
        ..Default::default()
    }])
}

fn shader_base_params_snapshot_tests() -> Vec<TestCase> {
    let input1 = TestInput::new(1);
    let input2 = TestInput::new(2);
    let input3 = TestInput::new(3);
    let input4 = TestInput::new(4);
    let input5 = TestInput::new(5);

    let plane_id_shader = (
        RendererId("base_params_plane_id".into()),
        RendererSpec::Shader(ShaderSpec {
            source: include_str!("../../snapshot_tests/shader/layout_planes.wgsl").into(),
        }),
    );

    let time_shader = (
        RendererId("base_params_time".into()),
        RendererSpec::Shader(ShaderSpec {
            source: include_str!("../../snapshot_tests/shader/fade_to_ball.wgsl").into(),
        }),
    );

    let texture_count_shader = (
        RendererId("base_params_texture_count".into()),
        RendererSpec::Shader(ShaderSpec {
            source: include_str!(
                "../../snapshot_tests/shader/color_output_with_texture_count.wgsl"
            )
            .into(),
        }),
    );

    let output_resolution_shader = (
        RendererId("base_params_output_resolution".into()),
        RendererSpec::Shader(ShaderSpec {
            source: include_str!("../../snapshot_tests/shader/red_border.wgsl").into(),
        }),
    );

    Vec::from([
        TestCase {
            name: "shader/base_params_plane_id_no_inputs",
            scene_updates: Updates::Scene(
                include_str!(
                    "../../snapshot_tests/shader/base_params_plane_id_no_inputs.scene.json"
                ),
                DEFAULT_RESOLUTION,
            ),
            renderers: vec![plane_id_shader.clone()],
            ..Default::default()
        },
        TestCase {
            name: "shader/base_params_plane_id_5_inputs",
            scene_updates: Updates::Scene(
                include_str!(
                    "../../snapshot_tests/shader/base_params_plane_id_5_inputs.scene.json"
                ),
                DEFAULT_RESOLUTION,
            ),
            renderers: vec![plane_id_shader.clone()],
            inputs: vec![
                input1.clone(),
                input2.clone(),
                input3.clone(),
                input4.clone(),
                input5.clone(),
            ],
            ..Default::default()
        },
        TestCase {
            name: "shader/base_params_time",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/shader/base_params_time.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            renderers: vec![time_shader.clone()],
            inputs: vec![input1.clone()],
            timestamps: vec![
                Duration::from_secs(0),
                Duration::from_secs(1),
                Duration::from_secs(2),
            ],
            ..Default::default()
        },
        TestCase {
            name: "shader/base_params_output_resolution",
            scene_updates: Updates::Scene(
                include_str!(
                    "../../snapshot_tests/shader/base_params_output_resolution.scene.json"
                ),
                DEFAULT_RESOLUTION,
            ),
            renderers: vec![output_resolution_shader.clone()],
            inputs: vec![input1.clone()],
            ..Default::default()
        },
        TestCase {
            name: "shader/base_params_texture_count_no_inputs",
            scene_updates: Updates::Scene(
                include_str!(
                    "../../snapshot_tests/shader/base_params_texture_count_no_inputs.scene.json"
                ),
                DEFAULT_RESOLUTION,
            ),
            renderers: vec![texture_count_shader.clone()],
            ..Default::default()
        },
        TestCase {
            name: "shader/base_params_texture_count_1_input",
            scene_updates: Updates::Scene(
                include_str!(
                    "../../snapshot_tests/shader/base_params_texture_count_1_input.scene.json"
                ),
                DEFAULT_RESOLUTION,
            ),
            renderers: vec![texture_count_shader.clone()],
            inputs: vec![input1.clone()],
            ..Default::default()
        },
        TestCase {
            name: "shader/base_params_texture_count_2_inputs",
            scene_updates: Updates::Scene(
                include_str!(
                    "../../snapshot_tests/shader/base_params_texture_count_2_inputs.scene.json"
                ),
                DEFAULT_RESOLUTION,
            ),
            renderers: vec![texture_count_shader.clone()],
            inputs: vec![input1.clone(), input2.clone()],
            ..Default::default()
        },
    ])
}

fn rescaler_snapshot_tests() -> Vec<TestCase> {
    let higher_than_default = Resolution {
        width: DEFAULT_RESOLUTION.width,
        height: DEFAULT_RESOLUTION.height + 100,
    };
    let lower_than_default = Resolution {
        width: DEFAULT_RESOLUTION.width,
        height: DEFAULT_RESOLUTION.height - 100,
    };
    let portrait_resolution = Resolution {
        width: 360,
        height: 640,
    };
    Vec::from([
        TestCase {
            name: "rescaler/fit_view_with_known_height",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/rescaler/fit_view_with_known_height.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            ..Default::default()
        },
        TestCase {
            name: "rescaler/fit_view_with_known_width",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/rescaler/fit_view_with_known_width.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            ..Default::default()
        },
        TestCase {
            name: "rescaler/fit_view_with_unknown_width_and_height",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/rescaler/fit_view_with_unknown_width_and_height.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            ..Default::default()
        },
        TestCase {
            name: "rescaler/fill_input_stream_inverted_aspect_ratio_align_top_left",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/rescaler/fill_input_stream_align_top_left.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new_with_resolution(1, portrait_resolution)],
            ..Default::default()
        },
        TestCase {
            name: "rescaler/fill_input_stream_inverted_aspect_ratio_align_bottom_right",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/rescaler/fill_input_stream_align_bottom_right.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new_with_resolution(1, portrait_resolution)],
            ..Default::default()
        },
        TestCase {
            name: "rescaler/fill_input_stream_lower_aspect_ratio_align_bottom_right",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/rescaler/fill_input_stream_align_bottom_right.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new_with_resolution(1, lower_than_default)],
            ..Default::default()
        },
        TestCase {
            name: "rescaler/fill_input_stream_lower_aspect_ratio",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/rescaler/fill_input_stream.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new_with_resolution(1, lower_than_default)],
            ..Default::default()
        },
        TestCase {
            name: "rescaler/fill_input_stream_higher_aspect_ratio",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/rescaler/fill_input_stream.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new_with_resolution(1, higher_than_default)],
            ..Default::default()
        },
        TestCase {
            name: "rescaler/fill_input_stream_inverted_aspect_ratio",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/rescaler/fill_input_stream.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new_with_resolution(1, portrait_resolution)],
            ..Default::default()
        },
        TestCase {
            name: "rescaler/fill_input_stream_matching_aspect_ratio",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/rescaler/fill_input_stream.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "rescaler/fit_input_stream_lower_aspect_ratio",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/rescaler/fit_input_stream.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new_with_resolution(1, lower_than_default)],
            ..Default::default()
        },
        TestCase {
            name: "rescaler/fit_input_stream_higher_aspect_ratio",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/rescaler/fit_input_stream.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new_with_resolution(1, higher_than_default)],
            ..Default::default()
        },
        TestCase {
            name: "rescaler/fit_input_stream_higher_aspect_ratio_small_resolution",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/rescaler/fit_input_stream.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new_with_resolution(1, Resolution { width: higher_than_default.width / 10, height: higher_than_default.height / 10 })],
            ..Default::default()
        },
        TestCase {
            name: "rescaler/fit_input_stream_inverted_aspect_ratio_align_top_left",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/rescaler/fit_input_stream_align_top_left.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new_with_resolution(1, portrait_resolution)],
            ..Default::default()
        },
        TestCase {
            name: "rescaler/fit_input_stream_inverted_aspect_ratio_align_bottom_right",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/rescaler/fit_input_stream_align_bottom_right.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new_with_resolution(1, portrait_resolution)],
            ..Default::default()
        },
        TestCase {
            name: "rescaler/fit_input_stream_lower_aspect_ratio_align_bottom_right",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/rescaler/fit_input_stream_align_bottom_right.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new_with_resolution(1, lower_than_default)],
            ..Default::default()
        },
        TestCase {
            name: "rescaler/fit_input_stream_inverted_aspect_ratio",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/rescaler/fit_input_stream.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new_with_resolution(1, portrait_resolution)],
            ..Default::default()
        },
        TestCase {
            name: "rescaler/fit_input_stream_matching_aspect_ratio",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/rescaler/fit_input_stream.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "rescaler/border_radius",
            scene_updates: Updates::Scene(
                    include_str!("../../snapshot_tests/rescaler/border_radius.scene.json"),
                    DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "rescaler/border_width",
            scene_updates: Updates::Scene(
                    include_str!("../../snapshot_tests/rescaler/border_width.scene.json"),
                    DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "rescaler/box_shadow",
            scene_updates: Updates::Scene(
                    include_str!("../../snapshot_tests/rescaler/box_shadow.scene.json"),
                    DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "rescaler/border_radius_border_box_shadow",
            scene_updates: Updates::Scene(
                    include_str!("../../snapshot_tests/rescaler/border_radius_border_box_shadow.scene.json"),
                    DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "rescaler/border_radius_box_shadow",
            scene_updates: Updates::Scene(
                    include_str!("../../snapshot_tests/rescaler/border_radius_box_shadow.scene.json"),
                    DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "rescaler/border_radius_box_shadow_fit_input_stream",
            scene_updates: Updates::Scene(
                    include_str!("../../snapshot_tests/rescaler/border_radius_box_shadow_fit_input_stream.scene.json"),
                    DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "rescaler/border_radius_box_shadow_fill_input_stream",
            scene_updates: Updates::Scene(
                    include_str!("../../snapshot_tests/rescaler/border_radius_box_shadow_fill_input_stream.scene.json"),
                    DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "rescaler/nested_border_width_radius",
            scene_updates: Updates::Scene(
                    include_str!("../../snapshot_tests/rescaler/nested_border_width_radius.scene.json"),
                    DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "rescaler/nested_border_width_radius_aligned",
            scene_updates: Updates::Scene(
                    include_str!("../../snapshot_tests/rescaler/nested_border_width_radius_aligned.scene.json"),
                    DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            // it is supposed to be cut off because of the rescaler that wraps it
            name: "rescaler/border_radius_border_box_shadow_rescaled",
            scene_updates: Updates::Scene(
                    include_str!("../../snapshot_tests/rescaler/border_radius_border_box_shadow_rescaled.scene.json"),
                    DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        }
    ])
}

fn tiles_snapshot_tests() -> Vec<TestCase> {
    let input1 = TestInput::new(1);
    let input2 = TestInput::new(2);
    let input3 = TestInput::new(3);
    let input4 = TestInput::new(4);
    let input5 = TestInput::new(5);
    let input6 = TestInput::new(6);
    let input7 = TestInput::new(7);
    let input8 = TestInput::new(8);
    let input9 = TestInput::new(9);
    let input10 = TestInput::new(10);
    let input11 = TestInput::new(11);
    let input12 = TestInput::new(12);
    let input13 = TestInput::new(13);
    let input14 = TestInput::new(14);
    let input15 = TestInput::new(15);
    let portrait_resolution = Resolution {
        width: 360,
        height: 640,
    };
    let portrait_input1 = TestInput::new_with_resolution(1, portrait_resolution);
    let portrait_input2 = TestInput::new_with_resolution(2, portrait_resolution);
    let portrait_input3 = TestInput::new_with_resolution(3, portrait_resolution);
    let portrait_input4 = TestInput::new_with_resolution(4, portrait_resolution);
    let portrait_input5 = TestInput::new_with_resolution(5, portrait_resolution);
    let portrait_input6 = TestInput::new_with_resolution(6, portrait_resolution);
    let portrait_input7 = TestInput::new_with_resolution(7, portrait_resolution);
    let portrait_input8 = TestInput::new_with_resolution(8, portrait_resolution);
    let portrait_input9 = TestInput::new_with_resolution(9, portrait_resolution);
    let portrait_input10 = TestInput::new_with_resolution(10, portrait_resolution);
    let portrait_input11 = TestInput::new_with_resolution(11, portrait_resolution);
    let portrait_input12 = TestInput::new_with_resolution(12, portrait_resolution);
    let portrait_input13 = TestInput::new_with_resolution(13, portrait_resolution);
    let portrait_input14 = TestInput::new_with_resolution(14, portrait_resolution);
    let portrait_input15 = TestInput::new_with_resolution(15, portrait_resolution);
    Vec::from([
        TestCase {
            name: "tiles_transitions/tile_resize_entire_component_with_parent_transition",
            scene_updates: Updates::Scenes(vec![
                (
                    include_str!("../../snapshot_tests/tiles_transitions/start_tile_resize.scene.json"),
                    DEFAULT_RESOLUTION,
                ),
                (
                    include_str!("../../snapshot_tests/tiles_transitions/end_tile_resize_with_view_transition.scene.json"),
                    DEFAULT_RESOLUTION,
                )
            ]),
            inputs: vec![
                input1.clone(),
                input2.clone(),
                input3.clone(),
            ],
            timestamps: vec![
                Duration::from_millis(0),
                Duration::from_millis(150),
                Duration::from_millis(350),
                // TODO: This transition does not look great, but it would require automatic
                // transitions triggered by a size change (not scene update)
                Duration::from_millis(450),
                Duration::from_millis(500),
            ],
            ..Default::default()
        },
        TestCase {
            name: "tiles_transitions/tile_resize_entire_component_without_parent_transition",
            scene_updates: Updates::Scenes(vec![
                (
                    include_str!("../../snapshot_tests/tiles_transitions/start_tile_resize.scene.json"),
                    DEFAULT_RESOLUTION,
                ),
                (
                    include_str!("../../snapshot_tests/tiles_transitions/end_tile_resize.scene.json"),
                    DEFAULT_RESOLUTION,
                )
            ]),
            inputs: vec![
                input1.clone(),
                input2.clone(),
                input3.clone(),
            ],
            timestamps: vec![
                Duration::from_millis(0),
                Duration::from_millis(150),
                Duration::from_millis(350),
                Duration::from_millis(500),
            ],
            ..Default::default()
        },
        TestCase {
            name: "tiles_transitions/change_order_of_3_inputs_with_id",
            scene_updates: Updates::Scenes(vec![
                (
                    include_str!("../../snapshot_tests/tiles_transitions/start_with_3_inputs_all_id.scene.json"),
                    DEFAULT_RESOLUTION,
                ),
                (
                    include_str!("../../snapshot_tests/tiles_transitions/end_with_3_inputs_3_id_different_order.scene.json"),
                    DEFAULT_RESOLUTION,
                )
            ]),
            inputs: vec![
                input1.clone(),
                input2.clone(),
                input3.clone(),
            ],
            timestamps: vec![
                Duration::from_millis(0),
                Duration::from_millis(150),
                Duration::from_millis(350),
                Duration::from_millis(500),
            ],
            ..Default::default()
        },
        TestCase {
            name: "tiles_transitions/replace_component_by_adding_id",
            scene_updates: Updates::Scenes(vec![
                (
                    include_str!("../../snapshot_tests/tiles_transitions/start_with_3_inputs_no_id.scene.json"),
                    DEFAULT_RESOLUTION,
                ),
                (
                    include_str!("../../snapshot_tests/tiles_transitions/end_with_3_inputs_1_id.scene.json"),
                    DEFAULT_RESOLUTION,
                )
            ]),
            inputs: vec![
                input1.clone(),
                input2.clone(),
                input3.clone(),
                input4.clone(),
            ],
            timestamps: vec![
                Duration::from_millis(0),
                Duration::from_millis(150),
                Duration::from_millis(350),
                Duration::from_millis(500),
            ],
            ..Default::default()
        },
        TestCase {
            name: "tiles_transitions/add_2_inputs_at_the_end_to_3_tiles_scene",
            scene_updates: Updates::Scenes(vec![
                (
                    include_str!("../../snapshot_tests/tiles_transitions/start_with_3_inputs_no_id.scene.json"),
                    DEFAULT_RESOLUTION,
                ),
                (
                    include_str!("../../snapshot_tests/tiles_transitions/end_with_5_inputs_no_id.scene.json"),
                    DEFAULT_RESOLUTION,
                )
            ]),
            inputs: vec![
                input1.clone(),
                input2.clone(),
                input3.clone(),
                input4.clone(),
                input5.clone(),
            ],
            timestamps: vec![
                Duration::from_millis(0),
                Duration::from_millis(150),
                Duration::from_millis(350),
                Duration::from_millis(500),
            ],
            ..Default::default()
        },
       TestCase {
            name: "tiles_transitions/add_input_on_2nd_pos_to_3_tiles_scene",
            scene_updates: Updates::Scenes(vec![
                (
                    include_str!("../../snapshot_tests/tiles_transitions/start_with_3_inputs_no_id.scene.json"),
                    DEFAULT_RESOLUTION,
                ),
                (
                    include_str!("../../snapshot_tests/tiles_transitions/end_with_4_inputs_1_id.scene.json"),
                    DEFAULT_RESOLUTION,
                )
            ]),
            inputs: vec![
                input1.clone(),
                input2.clone(),
                input3.clone(),
                input4.clone(),
            ],
            timestamps: vec![
                Duration::from_millis(0),
                Duration::from_millis(150),
                Duration::from_millis(350),
                Duration::from_millis(500),
            ],
            ..Default::default()
        },
        TestCase {
            name: "tiles_transitions/add_input_at_the_end_to_3_tiles_scene",
            scene_updates: Updates::Scenes(vec![
                (
                    include_str!("../../snapshot_tests/tiles_transitions/start_with_3_inputs_no_id.scene.json"),
                    DEFAULT_RESOLUTION,
                ),
                (
                    include_str!("../../snapshot_tests/tiles_transitions/end_with_4_inputs_no_id.scene.json"),
                    DEFAULT_RESOLUTION,
                ),
                (
                    include_str!("../../snapshot_tests/tiles_transitions/after_end_with_4_inputs_no_id.scene.json"),
                    DEFAULT_RESOLUTION,
                )
            ]),
            inputs: vec![
                input1.clone(),
                input2.clone(),
                input3.clone(),
                input4.clone(),
            ],
            timestamps: vec![
                Duration::from_millis(0),
                Duration::from_millis(150),
                Duration::from_millis(350),
                Duration::from_millis(500),
            ],
            ..Default::default()
        },
        TestCase {
            name: "tiles/01_inputs",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/tiles/01_inputs.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            inputs: vec![input1.clone()],
            ..Default::default()
        },
        TestCase {
            name: "tiles/02_inputs",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/tiles/02_inputs.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            inputs: vec![input1.clone(), input2.clone()],
            ..Default::default()
        },
        TestCase {
            name: "tiles/03_inputs",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/tiles/03_inputs.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            inputs: vec![input1.clone(), input2.clone(), input3.clone()],
            ..Default::default()
        },
        TestCase {
            name: "tiles/04_inputs",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/tiles/04_inputs.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            inputs: vec![
                input1.clone(),
                input2.clone(),
                input3.clone(),
                input4.clone(),
            ],
            ..Default::default()
        },
        TestCase {
            name: "tiles/05_inputs",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/tiles/05_inputs.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            inputs: vec![
                input1.clone(),
                input2.clone(),
                input3.clone(),
                input4.clone(),
                input5.clone(),
            ],
            ..Default::default()
        },
        TestCase {
            name: "tiles/15_inputs",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/tiles/15_inputs.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            inputs: vec![
                input1.clone(),
                input2.clone(),
                input3.clone(),
                input4.clone(),
                input5.clone(),
                input6.clone(),
                input7.clone(),
                input8.clone(),
                input9.clone(),
                input10.clone(),
                input11.clone(),
                input12.clone(),
                input13.clone(),
                input14.clone(),
                input15.clone(),
            ],
            ..Default::default()
        },
        TestCase {
            name: "tiles/01_portrait_inputs",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/tiles/01_portrait_inputs.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            inputs: vec![portrait_input1.clone()],
            ..Default::default()
        },
        TestCase {
            name: "tiles/02_portrait_inputs",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/tiles/02_portrait_inputs.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            inputs: vec![portrait_input1.clone(), portrait_input2.clone()],
            ..Default::default()
        },
        TestCase {
            name: "tiles/03_portrait_inputs",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/tiles/03_portrait_inputs.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            inputs: vec![
                portrait_input1.clone(),
                portrait_input2.clone(),
                portrait_input3.clone(),
            ],
            ..Default::default()
        },
        TestCase {
            name: "tiles/05_portrait_inputs",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/tiles/05_portrait_inputs.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            inputs: vec![
                portrait_input1.clone(),
                portrait_input2.clone(),
                portrait_input3.clone(),
                portrait_input4.clone(),
                portrait_input5.clone(),
            ],
            ..Default::default()
        },
        TestCase {
            name: "tiles/15_portrait_inputs",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/tiles/15_portrait_inputs.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            inputs: vec![
                portrait_input1.clone(),
                portrait_input2.clone(),
                portrait_input3.clone(),
                portrait_input4.clone(),
                portrait_input5.clone(),
                portrait_input6.clone(),
                portrait_input7.clone(),
                portrait_input8.clone(),
                portrait_input9.clone(),
                portrait_input10.clone(),
                portrait_input11.clone(),
                portrait_input12.clone(),
                portrait_input13.clone(),
                portrait_input14.clone(),
                portrait_input15.clone(),
            ],
            ..Default::default()
        },
        TestCase {
            name: "tiles/01_portrait_inputs_on_portrait_output",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/tiles/01_portrait_inputs.scene.json"),
                portrait_resolution,
            ),
            inputs: vec![portrait_input1.clone()],
            ..Default::default()
        },
        TestCase {
            name: "tiles/03_portrait_inputs_on_portrait_output",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/tiles/03_portrait_inputs.scene.json"),
                portrait_resolution,
            ),
            inputs: vec![
                portrait_input1.clone(),
                portrait_input2.clone(),
                portrait_input3.clone(),
            ],
            ..Default::default()
        },
        TestCase {
            name: "tiles/03_inputs_on_portrait_output",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/tiles/03_inputs.scene.json"),
                portrait_resolution,
            ),
            inputs: vec![input1.clone(), input2.clone(), input3.clone()],
            ..Default::default()
        },
        TestCase {
            name: "tiles/05_portrait_inputs_on_portrait_output",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/tiles/05_portrait_inputs.scene.json"),
                portrait_resolution,
            ),
            inputs: vec![
                portrait_input1.clone(),
                portrait_input2.clone(),
                portrait_input3.clone(),
                portrait_input4.clone(),
                portrait_input5.clone(),
            ],
            ..Default::default()
        },
        TestCase {
            name: "tiles/15_portrait_inputs_on_portrait_output",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/tiles/15_portrait_inputs.scene.json"),
                portrait_resolution,
            ),
            inputs: vec![
                portrait_input1.clone(),
                portrait_input2.clone(),
                portrait_input3.clone(),
                portrait_input4.clone(),
                portrait_input5.clone(),
                portrait_input6.clone(),
                portrait_input7.clone(),
                portrait_input8.clone(),
                portrait_input9.clone(),
                portrait_input10.clone(),
                portrait_input11.clone(),
                portrait_input12.clone(),
                portrait_input13.clone(),
                portrait_input14.clone(),
                portrait_input15.clone(),
            ],
            ..Default::default()
        },
        TestCase {
            name: "tiles/align_center_with_03_inputs",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/tiles/align_center_with_03_inputs.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            inputs: vec![input1.clone(), input2.clone(), input3.clone()],
            ..Default::default()
        },
        TestCase {
            name: "tiles/align_top_left_with_03_inputs",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/tiles/align_top_left_with_03_inputs.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            inputs: vec![input1.clone(), input2.clone(), input3.clone()],
            ..Default::default()
        },
        TestCase {
            name: "tiles/align_with_margin_and_padding_with_03_inputs",
            scene_updates: Updates::Scene(
                include_str!(
                "../../snapshot_tests/tiles/align_with_margin_and_padding_with_03_inputs.scene.json"
            ),
                DEFAULT_RESOLUTION,
            ),
            inputs: vec![input1.clone(), input2.clone(), input3.clone()],
            ..Default::default()
        },
        TestCase {
            name: "tiles/margin_with_03_inputs",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/tiles/margin_with_03_inputs.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            inputs: vec![input1.clone(), input2.clone(), input3.clone()],
            ..Default::default()
        },
        TestCase {
            name: "tiles/margin_and_padding_with_03_inputs",
            scene_updates: Updates::Scene(
                include_str!(
                    "../../snapshot_tests/tiles/margin_and_padding_with_03_inputs.scene.json"
                ),
                DEFAULT_RESOLUTION,
            ),
            inputs: vec![input1.clone(), input2.clone(), input3.clone()],
            ..Default::default()
        },
        TestCase {
            name: "tiles/padding_with_03_inputs",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/tiles/padding_with_03_inputs.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            inputs: vec![input1.clone(), input2.clone(), input3.clone()],
            ..Default::default()
        },
        TestCase{
            name: "tiles/video_call_with_labels",
            scene_updates: Updates::Scene(include_str!("../../snapshot_tests/tiles/video_call_with_labels.scene.json"), DEFAULT_RESOLUTION),
            inputs: vec![portrait_input1.clone(), portrait_input2.clone(), portrait_input3.clone()],
            ..Default::default()
        }
    ])
}

fn text_snapshot_tests() -> Vec<TestCase> {
    Vec::from([
        TestCase {
            name: "text/align_center",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/text/align_center.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            ..Default::default()
        },
        TestCase {
            name: "text/align_right",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/text/align_right.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            ..Default::default()
        },
        TestCase {
            name: "text/bold_text",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/text/bold_text.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            ..Default::default()
        },
        TestCase {
            name: "text/dimensions_fitted_column_with_long_text",
            scene_updates: Updates::Scene(
                include_str!(
                    "../../snapshot_tests/text/dimensions_fitted_column_with_long_text.scene.json"
                ),
                DEFAULT_RESOLUTION,
            ),
            ..Default::default()
        },
        TestCase {
            name: "text/dimensions_fitted_column_with_short_text",
            scene_updates: Updates::Scene(
                include_str!(
                    "../../snapshot_tests/text/dimensions_fitted_column_with_short_text.scene.json"
                ),
                DEFAULT_RESOLUTION,
            ),
            ..Default::default()
        },
        TestCase {
            name: "text/dimensions_fitted",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/text/dimensions_fitted.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            ..Default::default()
        },
        TestCase {
            name: "text/dimensions_fixed",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/text/dimensions_fixed.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            ..Default::default()
        },
        TestCase {
            name: "text/dimensions_fixed_with_overflow",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/text/dimensions_fixed_with_overflow.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            ..Default::default()
        },
        TestCase {
            name: "text/red_text_on_blue_background",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/text/red_text_on_blue_background.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            ..Default::default()
        },
        TestCase {
            name: "text/wrap_glyph",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/text/wrap_glyph.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            ..Default::default()
        },
        TestCase {
            name: "text/wrap_none",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/text/wrap_none.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            ..Default::default()
        },
        TestCase {
            name: "text/wrap_word",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/text/wrap_word.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            ..Default::default()
        },
        TestCase {
            // Test if removing text from scene works
            name: "text/remove_text_in_view",
            scene_updates: Updates::Scenes(vec![
                (
                    include_str!("../../snapshot_tests/text/align_center.scene.json"),
                    DEFAULT_RESOLUTION,
                ),
                (
                    include_str!("../../snapshot_tests/view/empty_view.scene.json"),
                    DEFAULT_RESOLUTION,
                ),
            ]),
            ..Default::default()
        },
        TestCase {
            // Test if removing text from scene works
            name: "text/remove_text_as_root",
            scene_updates: Updates::Scenes(vec![
                (
                    include_str!("../../snapshot_tests/text/root_text.scene.json"),
                    DEFAULT_RESOLUTION,
                ),
                (
                    include_str!("../../snapshot_tests/view/empty_view.scene.json"),
                    DEFAULT_RESOLUTION,
                ),
            ]),
            ..Default::default()
        },
        TestCase {
            name: "text/text_as_root",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/text/root_text.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            ..Default::default()
        },
    ])
}

fn image_snapshot_tests() -> Vec<TestCase> {
    let image_renderer = (
        RendererId("image_jpeg".into()),
        RendererSpec::Image(ImageSpec {
            src: ImageSource::Url {
                url: "https://www.rust-lang.org/static/images/rust-social.jpg".to_string(),
            },
            image_type: ImageType::Jpeg,
        }),
    );

    Vec::from([
        TestCase {
            name: "image/jpeg_as_root",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/image/jpeg_as_root.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            renderers: vec![image_renderer.clone()],
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "image/jpeg_in_view",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/image/jpeg_in_view.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            renderers: vec![image_renderer.clone()],
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "image/jpeg_in_view_overflow_fit",
            scene_updates: Updates::Scene(
                include_str!("../../snapshot_tests/image/jpeg_in_view_overflow_fit.scene.json"),
                DEFAULT_RESOLUTION,
            ),
            renderers: vec![image_renderer.clone()],
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            // Test if removing image from scene works
            name: "image/remove_jpeg_as_root",
            scene_updates: Updates::Scenes(vec![
                (
                    include_str!("../../snapshot_tests/image/jpeg_as_root.scene.json"),
                    DEFAULT_RESOLUTION,
                ),
                (
                    include_str!("../../snapshot_tests/view/empty_view.scene.json"),
                    DEFAULT_RESOLUTION,
                ),
            ]),
            renderers: vec![image_renderer.clone()],
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            // Test if removing image from scene works
            name: "image/remove_jpeg_in_view",
            scene_updates: Updates::Scenes(vec![
                (
                    include_str!("../../snapshot_tests/image/jpeg_in_view.scene.json"),
                    DEFAULT_RESOLUTION,
                ),
                (
                    include_str!("../../snapshot_tests/view/empty_view.scene.json"),
                    DEFAULT_RESOLUTION,
                ),
            ]),
            renderers: vec![image_renderer.clone()],
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
    ])
}

fn transition_snapshot_tests() -> Vec<TestCase> {
    Vec::from([
        TestCase {
            name: "transition/change_rescaler_absolute_and_send_next_update",
            scene_updates: Updates::Scenes(vec![
                (
                    include_str!(
                        "../../snapshot_tests/transition/change_rescaler_absolute_start.scene.json"
                    ),
                    DEFAULT_RESOLUTION,
                ),
                (
                    include_str!(
                        "../../snapshot_tests/transition/change_rescaler_absolute_end.scene.json"
                    ),
                    DEFAULT_RESOLUTION,
                ),
                (
                    include_str!(
                        "../../snapshot_tests/transition/change_rescaler_absolute_after_end.scene.json"
                    ),
                    DEFAULT_RESOLUTION,
                ),
            ]),
            timestamps: vec![
                Duration::from_secs(0),
                Duration::from_secs(5),
                Duration::from_secs(9),
                Duration::from_secs(10),
            ],
            ..Default::default()
        },
        TestCase {
            name: "transition/change_view_width_and_send_abort_transition",
            scene_updates: Updates::Scenes(vec![
                (
                    include_str!(
                        "../../snapshot_tests/transition/change_view_width_start.scene.json"
                    ),
                    DEFAULT_RESOLUTION,
                ),
                (
                    include_str!(
                        "../../snapshot_tests/transition/change_view_width_end.scene.json"
                    ),
                    DEFAULT_RESOLUTION,
                ),
                (
                    include_str!(
                        "../../snapshot_tests/transition/change_view_width_after_end_without_id.scene.json"
                    ),
                    DEFAULT_RESOLUTION,
                ),
            ]),
            timestamps: vec![
                Duration::from_secs(0),
                Duration::from_secs(5),
                Duration::from_secs(10),
            ],
            ..Default::default()
        },
        TestCase {
            name: "transition/change_view_width_and_send_next_update",
            scene_updates: Updates::Scenes(vec![
                (
                    include_str!(
                        "../../snapshot_tests/transition/change_view_width_start.scene.json"
                    ),
                    DEFAULT_RESOLUTION,
                ),
                (
                    include_str!(
                        "../../snapshot_tests/transition/change_view_width_end.scene.json"
                    ),
                    DEFAULT_RESOLUTION,
                ),
                (
                    include_str!(
                        "../../snapshot_tests/transition/change_view_width_after_end.scene.json"
                    ),
                    DEFAULT_RESOLUTION,
                ),
            ]),
            timestamps: vec![
                Duration::from_secs(0),
                Duration::from_secs(5),
                Duration::from_secs(10),
            ],
            ..Default::default()
        },
        TestCase {
            name: "transition/change_view_width",
            scene_updates: Updates::Scenes(vec![
                (
                    include_str!(
                        "../../snapshot_tests/transition/change_view_width_start.scene.json"
                    ),
                    DEFAULT_RESOLUTION,
                ),
                (
                    include_str!(
                        "../../snapshot_tests/transition/change_view_width_end.scene.json"
                    ),
                    DEFAULT_RESOLUTION,
                ),
            ]),
            timestamps: vec![
                Duration::from_secs(0),
                Duration::from_secs(5),
                Duration::from_secs(10),
                Duration::from_secs(100),
            ],
            ..Default::default()
        },
        TestCase {
            name: "transition/change_view_height",
            scene_updates: Updates::Scenes(vec![
                (
                    include_str!(
                        "../../snapshot_tests/transition/change_view_height_start.scene.json"
                    ),
                    DEFAULT_RESOLUTION,
                ),
                (
                    include_str!(
                        "../../snapshot_tests/transition/change_view_height_end.scene.json"
                    ),
                    DEFAULT_RESOLUTION,
                ),
            ]),
            timestamps: vec![
                Duration::from_secs(0),
                Duration::from_secs(5),
                Duration::from_secs(10),
            ],
            ..Default::default()
        },
        TestCase {
            name: "transition/change_view_absolute",
            scene_updates: Updates::Scenes(vec![
                (
                    include_str!(
                        "../../snapshot_tests/transition/change_view_absolute_start.scene.json"
                    ),
                    DEFAULT_RESOLUTION,
                ),
                (
                    include_str!(
                        "../../snapshot_tests/transition/change_view_absolute_end.scene.json"
                    ),
                    DEFAULT_RESOLUTION,
                ),
            ]),
            timestamps: vec![
                Duration::from_secs(0),
                Duration::from_secs(5),
                Duration::from_secs(9),
                Duration::from_secs(10),
            ],
            ..Default::default()
        },
        TestCase {
            name: "transition/change_view_absolute_cubic_bezier",
            scene_updates: Updates::Scenes(vec![
                (
                    include_str!(
                        "../../snapshot_tests/transition/change_view_absolute_cubic_bezier_start.scene.json"
                    ),
                    DEFAULT_RESOLUTION,
                ),
                (
                    include_str!(
                        "../../snapshot_tests/transition/change_view_absolute_cubic_bezier_end.scene.json"
                    ),
                    DEFAULT_RESOLUTION,
                ),
            ]),
            timestamps: vec![
                Duration::from_millis(0),
                Duration::from_millis(2500),
                Duration::from_secs(5000),
            ],
            ..Default::default()
        },
        TestCase {
            name: "transition/change_view_absolute_cubic_bezier_linear_like",
            scene_updates: Updates::Scenes(vec![
                (
                    include_str!(
                        "../../snapshot_tests/transition/change_view_absolute_cubic_bezier_linear_like_start.scene.json"
                    ),
                    DEFAULT_RESOLUTION,
                ),
                (
                    include_str!(
                        "../../snapshot_tests/transition/change_view_absolute_cubic_bezier_linear_like_end.scene.json"
                    ),
                    DEFAULT_RESOLUTION,
                ),
            ]),
            timestamps: vec![
                Duration::from_millis(0),
                Duration::from_millis(2500),
                Duration::from_secs(5000),
            ],
            ..Default::default()
        },
    ])
}

fn view_snapshot_tests() -> Vec<TestCase> {
    Vec::from([
        TestCase {
            name: "view/overflow_hidden_with_input_stream_children",
            scene_updates: Updates::Scene(
                    include_str!("../../snapshot_tests/view/overflow_hidden_with_input_stream_children.scene.json"),
                    DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new_with_resolution(1, Resolution { width: 180, height: 200 })],
            ..Default::default()
        },
        TestCase {
            name: "view/overflow_hidden_with_view_children",
            scene_updates: Updates::Scene(
                    include_str!("../../snapshot_tests/view/overflow_hidden_with_view_children.scene.json"),
                    DEFAULT_RESOLUTION,
            ),
            inputs: vec![],
            ..Default::default()
        },
        TestCase {
            name: "view/constant_width_views_row",
            scene_updates: Updates::Scene(
                    include_str!("../../snapshot_tests/view/constant_width_views_row.scene.json"),
                    DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "view/constant_width_views_row_with_overflow_hidden",
            scene_updates: Updates::Scene(
                    include_str!("../../snapshot_tests/view/constant_width_views_row_with_overflow_hidden.scene.json"),
                    DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "view/constant_width_views_row_with_overflow_visible",
            scene_updates: Updates::Scene(
                    include_str!("../../snapshot_tests/view/constant_width_views_row_with_overflow_visible.scene.json"),
                    DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "view/constant_width_views_row_with_overflow_fit",
            scene_updates: Updates::Scene(
                    include_str!("../../snapshot_tests/view/constant_width_views_row_with_overflow_fit.scene.json"),
                    DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "view/dynamic_width_views_row",
            scene_updates: Updates::Scene(
                    include_str!("../../snapshot_tests/view/dynamic_width_views_row.scene.json"),
                    DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "view/dynamic_and_constant_width_views_row",
            scene_updates: Updates::Scene(
                    include_str!("../../snapshot_tests/view/dynamic_and_constant_width_views_row.scene.json"),
                    DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "view/dynamic_and_constant_width_views_row_with_overflow",
            scene_updates: Updates::Scene(
                    include_str!("../../snapshot_tests/view/dynamic_and_constant_width_views_row_with_overflow.scene.json"),
                    DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "view/constant_width_and_height_views_row",
            scene_updates: Updates::Scene(
                    include_str!("../../snapshot_tests/view/constant_width_and_height_views_row.scene.json"),
                    DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "view/view_with_absolute_positioning_partially_covered_by_sibling",
            scene_updates: Updates::Scene(
                    include_str!("../../snapshot_tests/view/view_with_absolute_positioning_partially_covered_by_sibling.scene.json"),
                    DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "view/view_with_absolute_positioning_render_over_siblings",
            scene_updates: Updates::Scene(
                    include_str!("../../snapshot_tests/view/view_with_absolute_positioning_render_over_siblings.scene.json"),
                    DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "view/root_view_with_background_color",
            scene_updates: Updates::Scene(
                    include_str!("../../snapshot_tests/view/root_view_with_background_color.scene.json"),
                    DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "view/border_radius",
            scene_updates: Updates::Scene(
                    include_str!("../../snapshot_tests/view/border_radius.scene.json"),
                    DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "view/border_width",
            scene_updates: Updates::Scene(
                    include_str!("../../snapshot_tests/view/border_width.scene.json"),
                    DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "view/box_shadow",
            scene_updates: Updates::Scene(
                    include_str!("../../snapshot_tests/view/box_shadow.scene.json"),
                    DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "view/box_shadow_sibling",
            scene_updates: Updates::Scene(
                    include_str!("../../snapshot_tests/view/box_shadow_sibling.scene.json"),
                    DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "view/border_radius_border_box_shadow",
            scene_updates: Updates::Scene(
                    include_str!("../../snapshot_tests/view/border_radius_border_box_shadow.scene.json"),
                    DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "view/border_radius_box_shadow",
            scene_updates: Updates::Scene(
                    include_str!("../../snapshot_tests/view/border_radius_box_shadow.scene.json"),
                    DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "view/border_radius_box_shadow_overflow_hidden",
            scene_updates: Updates::Scene(
                    include_str!("../../snapshot_tests/view/border_radius_box_shadow_overflow_hidden.scene.json"),
                    DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "view/border_radius_box_shadow_overflow_fit",
            scene_updates: Updates::Scene(
                    include_str!("../../snapshot_tests/view/border_radius_box_shadow_overflow_fit.scene.json"),
                    DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "view/border_radius_box_shadow_rescaler_input_stream",
            scene_updates: Updates::Scene(
                    include_str!("../../snapshot_tests/view/border_radius_box_shadow_rescaler_input_stream.scene.json"),
                    DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "view/nested_border_width_radius",
            scene_updates: Updates::Scene(
                    include_str!("../../snapshot_tests/view/nested_border_width_radius.scene.json"),
                    DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "view/nested_border_width_radius_aligned",
            scene_updates: Updates::Scene(
                    include_str!("../../snapshot_tests/view/nested_border_width_radius_aligned.scene.json"),
                    DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "view/nested_border_width_radius_multi_child",
            scene_updates: Updates::Scene(
                    include_str!("../../snapshot_tests/view/nested_border_width_radius_multi_child.scene.json"),
                    DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            // it is supposed to be cut off because of the rescaler that wraps it
            name: "view/border_radius_border_box_shadow_rescaled",
            scene_updates: Updates::Scene(
                    include_str!("../../snapshot_tests/view/border_radius_border_box_shadow_rescaled.scene.json"),
                    DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "view/root_border_radius_border_box_shadow",
            scene_updates: Updates::Scene(
                    include_str!("../../snapshot_tests/view/root_border_radius_border_box_shadow.scene.json"),
                    DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "view/border_radius_border_box_shadow_rescaled_and_hidden_by_parent",
            scene_updates: Updates::Scene(
                    include_str!("../../snapshot_tests/view/border_radius_border_box_shadow_rescaled_and_hidden_by_parent.scene.json"),
                    DEFAULT_RESOLUTION,
            ),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
    ])
}

fn base_snapshot_tests() -> Vec<TestCase> {
    Vec::from([TestCase {
        name: "simple_input_pass_through",
        scene_updates: Updates::Scene(
            include_str!("../../snapshot_tests/simple_input_pass_through.scene.json"),
            DEFAULT_RESOLUTION,
        ),
        inputs: vec![TestInput::new(1)],
        ..Default::default()
    }])
}
