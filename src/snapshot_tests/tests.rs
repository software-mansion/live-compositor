use std::time::Duration;

use compositor_render::Resolution;
use serde_json::json;

use super::test_case::{Outputs, TestCase, TestInput};

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
    let input1 = TestInput::new(1);
    let input2 = TestInput::new(2);
    let input3 = TestInput::new(3);
    let input4 = TestInput::new(4);
    let input5 = TestInput::new(5);

    let plane_id_shader = Box::new(json!({
            "type": "register",
            "entity_type": "shader",
            "shader_id": "base_params_plane_id",
            "source": include_str!("../../snapshot_tests/shader/layout_planes.wgsl")
        })
        .to_string());

    let time_shader = Box::new(json!({
            "type": "register",
            "entity_type": "shader",
            "shader_id": "base_params_time",
            "source": include_str!("../../snapshot_tests/shader/fade_to_ball.wgsl")
        })
        .to_string());

    let texture_count_shader = Box::new(json!({
            "type": "register",
            "entity_type": "shader",
            "shader_id": "base_params_texture_count",
            "source": include_str!("../../snapshot_tests/shader/color_output_with_texture_count.wgsl")
        })
        .to_string());

    let output_resolution_shader = Box::new(json!({
            "type": "register",
            "entity_type": "shader",
            "shader_id": "base_params_output_resolution",
            "source": include_str!("../../snapshot_tests/shader/red_border.wgsl")
        })
        .to_string());

    Vec::from([
        TestCase {
            name: "shader/base_params_plane_id_no_inputs",
            outputs: Outputs::Scene(vec![(
                include_str!(
                    "../../snapshot_tests/shader/base_params_plane_id_no_inputs.scene.json"
                ),
                DEFAULT_RESOLUTION,
            )]),
            renderers: vec![plane_id_shader.clone().leak()],
            ..Default::default()
        },
        TestCase {
            name: "shader/base_params_plane_id_5_inputs",
            outputs: Outputs::Scene(vec![(
                include_str!(
                    "../../snapshot_tests/shader/base_params_plane_id_5_inputs.scene.json"
                ),
                DEFAULT_RESOLUTION,
            )]),
            renderers: vec![plane_id_shader.clone().leak()],
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
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/shader/base_params_time.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
            renderers: vec![time_shader.clone().leak()],
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
            outputs: Outputs::Scene(vec![(
                include_str!(
                    "../../snapshot_tests/shader/base_params_output_resolution.scene.json"
                ),
                DEFAULT_RESOLUTION,
            )]),
            renderers: vec![output_resolution_shader.clone().leak()],
            inputs: vec![input1.clone()],
            ..Default::default()
        },
        TestCase {
            name: "shader/base_params_texture_count_no_inputs",
            outputs: Outputs::Scene(vec![(
                include_str!(
                    "../../snapshot_tests/shader/base_params_texture_count_no_inputs.scene.json"
                ),
                DEFAULT_RESOLUTION,
            )]),
            renderers: vec![texture_count_shader.clone().leak()],
            ..Default::default()
        },
        TestCase {
            name: "shader/base_params_texture_count_1_input",
            outputs: Outputs::Scene(vec![(
                include_str!(
                    "../../snapshot_tests/shader/base_params_texture_count_1_input.scene.json"
                ),
                DEFAULT_RESOLUTION,
            )]),
            renderers: vec![texture_count_shader.clone().leak()],
            inputs: vec![input1.clone()],
            ..Default::default()
        },
        TestCase {
            name: "shader/base_params_texture_count_2_inputs",
            outputs: Outputs::Scene(vec![(
                include_str!(
                    "../../snapshot_tests/shader/base_params_texture_count_2_inputs.scene.json"
                ),
                DEFAULT_RESOLUTION,
            )]),
            renderers: vec![texture_count_shader.clone().leak()],
            inputs: vec![input1.clone(), input2.clone()],
            ..Default::default()
        }
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
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/rescaler/fit_view_with_known_height.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
            ..Default::default()
        },
        TestCase {
            name: "rescaler/fit_view_with_known_width",
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/rescaler/fit_view_with_known_width.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
            ..Default::default()
        },
        TestCase {
            name: "rescaler/fit_view_with_unknown_width_and_height",
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/rescaler/fit_view_with_unknown_width_and_height.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
            ..Default::default()
        },
        TestCase {
            name: "rescaler/fill_input_stream_inverted_aspect_ratio_align_top_left",
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/rescaler/fill_input_stream_align_top_left.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
            inputs: vec![TestInput::new_with_resolution(1, portrait_resolution)],
            ..Default::default()
        },
        TestCase {
            name: "rescaler/fill_input_stream_inverted_aspect_ratio_align_bottom_right",
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/rescaler/fill_input_stream_align_bottom_right.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
            inputs: vec![TestInput::new_with_resolution(1, portrait_resolution)],
            ..Default::default()
        },
        TestCase {
            name: "rescaler/fill_input_stream_lower_aspect_ratio_align_bottom_right",
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/rescaler/fill_input_stream_align_bottom_right.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
            inputs: vec![TestInput::new_with_resolution(1, lower_than_default)],
            ..Default::default()
        },
        TestCase {
            name: "rescaler/fill_input_stream_lower_aspect_ratio",
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/rescaler/fill_input_stream.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
            inputs: vec![TestInput::new_with_resolution(1, lower_than_default)],
            ..Default::default()
        },
        TestCase {
            name: "rescaler/fill_input_stream_higher_aspect_ratio",
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/rescaler/fill_input_stream.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
            inputs: vec![TestInput::new_with_resolution(1, higher_than_default)],
            ..Default::default()
        },
        TestCase {
            name: "rescaler/fill_input_stream_inverted_aspect_ratio",
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/rescaler/fill_input_stream.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
            inputs: vec![TestInput::new_with_resolution(1, portrait_resolution)],
            ..Default::default()
        },
        TestCase {
            name: "rescaler/fill_input_stream_matching_aspect_ratio",
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/rescaler/fill_input_stream.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "rescaler/fit_input_stream_lower_aspect_ratio",
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/rescaler/fit_input_stream.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
            inputs: vec![TestInput::new_with_resolution(1, lower_than_default)],
            ..Default::default()
        },
        TestCase {
            name: "rescaler/fit_input_stream_higher_aspect_ratio",
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/rescaler/fit_input_stream.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
            inputs: vec![TestInput::new_with_resolution(1, higher_than_default)],
            ..Default::default()
        },
        TestCase {
            name: "rescaler/fit_input_stream_higher_aspect_ratio_small_resolution",
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/rescaler/fit_input_stream.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
            inputs: vec![TestInput::new_with_resolution(1, Resolution { width: higher_than_default.width / 10, height: higher_than_default.height / 10 })],
            ..Default::default()
        },
        TestCase {
            name: "rescaler/fit_input_stream_inverted_aspect_ratio_align_top_left",
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/rescaler/fit_input_stream_align_top_left.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
            inputs: vec![TestInput::new_with_resolution(1, portrait_resolution)],
            ..Default::default()
        },
        TestCase {
            name: "rescaler/fit_input_stream_inverted_aspect_ratio_align_bottom_right",
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/rescaler/fit_input_stream_align_bottom_right.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
            inputs: vec![TestInput::new_with_resolution(1, portrait_resolution)],
            ..Default::default()
        },
        TestCase {
            name: "rescaler/fit_input_stream_lower_aspect_ratio_align_bottom_right",
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/rescaler/fit_input_stream_align_bottom_right.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
            inputs: vec![TestInput::new_with_resolution(1, lower_than_default)],
            ..Default::default()
        },
        TestCase {
            name: "rescaler/fit_input_stream_inverted_aspect_ratio",
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/rescaler/fit_input_stream.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
            inputs: vec![TestInput::new_with_resolution(1, portrait_resolution)],
            ..Default::default()
        },
        TestCase {
            name: "rescaler/fit_input_stream_matching_aspect_ratio",
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/rescaler/fit_input_stream.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
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
            outputs: Outputs::Scenes(vec![
                vec![(
                    include_str!("../../snapshot_tests/tiles_transitions/start_tile_resize.scene.json"),
                    DEFAULT_RESOLUTION,
                )],
                vec![(
                    include_str!("../../snapshot_tests/tiles_transitions/end_tile_resize_with_view_transition.scene.json"),
                    DEFAULT_RESOLUTION,
                )]
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
            outputs: Outputs::Scenes(vec![
                vec![(
                    include_str!("../../snapshot_tests/tiles_transitions/start_tile_resize.scene.json"),
                    DEFAULT_RESOLUTION,
                )],
                vec![(
                    include_str!("../../snapshot_tests/tiles_transitions/end_tile_resize.scene.json"),
                    DEFAULT_RESOLUTION,
                )]
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
            outputs: Outputs::Scenes(vec![
                vec![(
                    include_str!("../../snapshot_tests/tiles_transitions/start_with_3_inputs_all_id.scene.json"),
                    DEFAULT_RESOLUTION,
                )],
                vec![(
                    include_str!("../../snapshot_tests/tiles_transitions/end_with_3_inputs_3_id_different_order.scene.json"),
                    DEFAULT_RESOLUTION,
                )]
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
            outputs: Outputs::Scenes(vec![
                vec![(
                    include_str!("../../snapshot_tests/tiles_transitions/start_with_3_inputs_no_id.scene.json"),
                    DEFAULT_RESOLUTION,
                )],
                vec![(
                    include_str!("../../snapshot_tests/tiles_transitions/end_with_3_inputs_1_id.scene.json"),
                    DEFAULT_RESOLUTION,
                )]
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
            outputs: Outputs::Scenes(vec![
                vec![(
                    include_str!("../../snapshot_tests/tiles_transitions/start_with_3_inputs_no_id.scene.json"),
                    DEFAULT_RESOLUTION,
                )],
                vec![(
                    include_str!("../../snapshot_tests/tiles_transitions/end_with_5_inputs_no_id.scene.json"),
                    DEFAULT_RESOLUTION,
                )]
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
            outputs: Outputs::Scenes(vec![
                vec![(
                    include_str!("../../snapshot_tests/tiles_transitions/start_with_3_inputs_no_id.scene.json"),
                    DEFAULT_RESOLUTION,
                )],
                vec![(
                    include_str!("../../snapshot_tests/tiles_transitions/end_with_4_inputs_1_id.scene.json"),
                    DEFAULT_RESOLUTION,
                )]
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
            outputs: Outputs::Scenes(vec![
                vec![(
                    include_str!("../../snapshot_tests/tiles_transitions/start_with_3_inputs_no_id.scene.json"),
                    DEFAULT_RESOLUTION,
                )],
                vec![(
                    include_str!("../../snapshot_tests/tiles_transitions/end_with_4_inputs_no_id.scene.json"),
                    DEFAULT_RESOLUTION,
                )],
                vec![(
                    include_str!("../../snapshot_tests/tiles_transitions/after_end_with_4_inputs_no_id.scene.json"),
                    DEFAULT_RESOLUTION,
                )]
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
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/tiles/01_inputs.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
            inputs: vec![input1.clone()],
            ..Default::default()
        },
        TestCase {
            name: "tiles/02_inputs",
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/tiles/02_inputs.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
            inputs: vec![input1.clone(), input2.clone()],
            ..Default::default()
        },
        TestCase {
            name: "tiles/03_inputs",
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/tiles/03_inputs.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
            inputs: vec![input1.clone(), input2.clone(), input3.clone()],
            ..Default::default()
        },
        TestCase {
            name: "tiles/04_inputs",
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/tiles/04_inputs.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
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
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/tiles/05_inputs.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
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
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/tiles/15_inputs.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
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
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/tiles/01_portrait_inputs.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
            inputs: vec![portrait_input1.clone()],
            ..Default::default()
        },
        TestCase {
            name: "tiles/02_portrait_inputs",
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/tiles/02_portrait_inputs.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
            inputs: vec![portrait_input1.clone(), portrait_input2.clone()],
            ..Default::default()
        },
        TestCase {
            name: "tiles/03_portrait_inputs",
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/tiles/03_portrait_inputs.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
            inputs: vec![
                portrait_input1.clone(),
                portrait_input2.clone(),
                portrait_input3.clone(),
            ],
            ..Default::default()
        },
        TestCase {
            name: "tiles/05_portrait_inputs",
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/tiles/05_portrait_inputs.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
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
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/tiles/15_portrait_inputs.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
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
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/tiles/01_portrait_inputs.scene.json"),
                portrait_resolution,
            )]),
            inputs: vec![portrait_input1.clone()],
            ..Default::default()
        },
        TestCase {
            name: "tiles/03_portrait_inputs_on_portrait_output",
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/tiles/03_portrait_inputs.scene.json"),
                portrait_resolution,
            )]),
            inputs: vec![
                portrait_input1.clone(),
                portrait_input2.clone(),
                portrait_input3.clone(),
            ],
            ..Default::default()
        },
        TestCase {
            name: "tiles/03_inputs_on_portrait_output",
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/tiles/03_inputs.scene.json"),
                portrait_resolution,
            )]),
            inputs: vec![input1.clone(), input2.clone(), input3.clone()],
            ..Default::default()
        },
        TestCase {
            name: "tiles/05_portrait_inputs_on_portrait_output",
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/tiles/05_portrait_inputs.scene.json"),
                portrait_resolution,
            )]),
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
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/tiles/15_portrait_inputs.scene.json"),
                portrait_resolution,
            )]),
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
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/tiles/align_center_with_03_inputs.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
            inputs: vec![input1.clone(), input2.clone(), input3.clone()],
            ..Default::default()
        },
        TestCase {
            name: "tiles/align_top_left_with_03_inputs",
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/tiles/align_top_left_with_03_inputs.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
            inputs: vec![input1.clone(), input2.clone(), input3.clone()],
            ..Default::default()
        },
        TestCase {
            name: "tiles/align_with_margin_and_padding_with_03_inputs",
            outputs: Outputs::Scene(vec![(
                include_str!(
                "../../snapshot_tests/tiles/align_with_margin_and_padding_with_03_inputs.scene.json"
            ),
                DEFAULT_RESOLUTION,
            )]),
            inputs: vec![input1.clone(), input2.clone(), input3.clone()],
            ..Default::default()
        },
        TestCase {
            name: "tiles/margin_with_03_inputs",
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/tiles/margin_with_03_inputs.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
            inputs: vec![input1.clone(), input2.clone(), input3.clone()],
            ..Default::default()
        },
        TestCase {
            name: "tiles/margin_and_padding_with_03_inputs",
            outputs: Outputs::Scene(vec![(
                include_str!(
                    "../../snapshot_tests/tiles/margin_and_padding_with_03_inputs.scene.json"
                ),
                DEFAULT_RESOLUTION,
            )]),
            inputs: vec![input1.clone(), input2.clone(), input3.clone()],
            ..Default::default()
        },
        TestCase {
            name: "tiles/padding_with_03_inputs",
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/tiles/padding_with_03_inputs.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
            inputs: vec![input1.clone(), input2.clone(), input3.clone()],
            ..Default::default()
        },
    ])
}

fn text_snapshot_tests() -> Vec<TestCase> {
    Vec::from([
        TestCase {
            name: "text/align_center",
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/text/align_center.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
            ..Default::default()
        },
        TestCase {
            name: "text/align_right",
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/text/align_right.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
            ..Default::default()
        },
        TestCase {
            name: "text/bold_text",
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/text/bold_text.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
            ..Default::default()
        },
        TestCase {
            name: "text/dimensions_fitted_column_with_long_text",
            outputs: Outputs::Scene(vec![(
                include_str!(
                    "../../snapshot_tests/text/dimensions_fitted_column_with_long_text.scene.json"
                ),
                DEFAULT_RESOLUTION,
            )]),
            ..Default::default()
        },
        TestCase {
            name: "text/dimensions_fitted_column_with_short_text",
            outputs: Outputs::Scene(vec![(
                include_str!(
                    "../../snapshot_tests/text/dimensions_fitted_column_with_short_text.scene.json"
                ),
                DEFAULT_RESOLUTION,
            )]),
            ..Default::default()
        },
        TestCase {
            name: "text/dimensions_fitted",
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/text/dimensions_fitted.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
            ..Default::default()
        },
        TestCase {
            name: "text/dimensions_fixed",
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/text/dimensions_fixed.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
            ..Default::default()
        },
        TestCase {
            name: "text/dimensions_fixed_with_overflow",
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/text/dimensions_fixed_with_overflow.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
            ..Default::default()
        },
        TestCase {
            name: "text/red_text_on_blue_background",
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/text/red_text_on_blue_background.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
            ..Default::default()
        },
        TestCase {
            name: "text/wrap_glyph",
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/text/wrap_glyph.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
            allowed_error: 325.7,
            ..Default::default()
        },
        TestCase {
            name: "text/wrap_none",
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/text/wrap_none.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
            ..Default::default()
        },
        TestCase {
            name: "text/wrap_word",
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/text/wrap_word.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
            allowed_error: 321.8,
            ..Default::default()
        },
    ])
}

fn image_snapshot_tests() -> Vec<TestCase> {
    let image_renderer = include_str!("../../snapshot_tests/register/image_jpeg.register.json");

    Vec::from([
        TestCase {
            name: "image/jpeg_as_root",
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/image/jpeg_as_root.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
            renderers: vec![image_renderer],
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "image/jpeg_in_view",
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/image/jpeg_in_view.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
            renderers: vec![image_renderer],
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "image/jpeg_in_view_overflow_fit",
            outputs: Outputs::Scene(vec![(
                include_str!("../../snapshot_tests/image/jpeg_in_view_overflow_fit.scene.json"),
                DEFAULT_RESOLUTION,
            )]),
            renderers: vec![image_renderer],
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
    ])
}

fn transition_snapshot_tests() -> Vec<TestCase> {
    Vec::from([
        TestCase {
            name: "transition/change_rescaler_absolute_and_send_next_update",
            outputs: Outputs::Scenes(vec![
                vec![(
                    include_str!(
                        "../../snapshot_tests/transition/change_rescaler_absolute_start.scene.json"
                    ),
                    DEFAULT_RESOLUTION,
                )],
                vec![(
                    include_str!(
                        "../../snapshot_tests/transition/change_rescaler_absolute_end.scene.json"
                    ),
                    DEFAULT_RESOLUTION,
                )],
                vec![(
                    include_str!(
                        "../../snapshot_tests/transition/change_rescaler_absolute_after_end.scene.json"
                    ),
                    DEFAULT_RESOLUTION,
                )],
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
            outputs: Outputs::Scenes(vec![
                vec![(
                    include_str!(
                        "../../snapshot_tests/transition/change_view_width_start.scene.json"
                    ),
                    DEFAULT_RESOLUTION,
                )],
                vec![(
                    include_str!(
                        "../../snapshot_tests/transition/change_view_width_end.scene.json"
                    ),
                    DEFAULT_RESOLUTION,
                )],
                vec![(
                    include_str!(
                        "../../snapshot_tests/transition/change_view_width_after_end_without_id.scene.json"
                    ),
                    DEFAULT_RESOLUTION,
                )],
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
            outputs: Outputs::Scenes(vec![
                vec![(
                    include_str!(
                        "../../snapshot_tests/transition/change_view_width_start.scene.json"
                    ),
                    DEFAULT_RESOLUTION,
                )],
                vec![(
                    include_str!(
                        "../../snapshot_tests/transition/change_view_width_end.scene.json"
                    ),
                    DEFAULT_RESOLUTION,
                )],
                vec![(
                    include_str!(
                        "../../snapshot_tests/transition/change_view_width_after_end.scene.json"
                    ),
                    DEFAULT_RESOLUTION,
                )],
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
            outputs: Outputs::Scenes(vec![
                vec![(
                    include_str!(
                        "../../snapshot_tests/transition/change_view_width_start.scene.json"
                    ),
                    DEFAULT_RESOLUTION,
                )],
                vec![(
                    include_str!(
                        "../../snapshot_tests/transition/change_view_width_end.scene.json"
                    ),
                    DEFAULT_RESOLUTION,
                )],
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
            outputs: Outputs::Scenes(vec![
                vec![(
                    include_str!(
                        "../../snapshot_tests/transition/change_view_height_start.scene.json"
                    ),
                    DEFAULT_RESOLUTION,
                )],
                vec![(
                    include_str!(
                        "../../snapshot_tests/transition/change_view_height_end.scene.json"
                    ),
                    DEFAULT_RESOLUTION,
                )],
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
            outputs: Outputs::Scenes(vec![
                vec![(
                    include_str!(
                        "../../snapshot_tests/transition/change_view_absolute_start.scene.json"
                    ),
                    DEFAULT_RESOLUTION,
                )],
                vec![(
                    include_str!(
                        "../../snapshot_tests/transition/change_view_absolute_end.scene.json"
                    ),
                    DEFAULT_RESOLUTION,
                )],
            ]),
            timestamps: vec![
                Duration::from_secs(0),
                Duration::from_secs(5),
                Duration::from_secs(9),
                Duration::from_secs(10),
            ],
            ..Default::default()
        },
    ])
}

fn view_snapshot_tests() -> Vec<TestCase> {
    Vec::from([
        TestCase {
            name: "view/overflow_hidden_with_input_stream_children",
            outputs: Outputs::Scene(vec![(
                    include_str!("../../snapshot_tests/view/overflow_hidden_with_input_stream_children.scene.json"),
                    DEFAULT_RESOLUTION,
            )]),
            inputs: vec![TestInput::new_with_resolution(1, Resolution { width: 180, height: 200 })],
            ..Default::default()
        },
        TestCase {
            name: "view/overflow_hidden_with_view_children",
            outputs: Outputs::Scene(vec![(
                    include_str!("../../snapshot_tests/view/overflow_hidden_with_view_children.scene.json"),
                    DEFAULT_RESOLUTION,
            )]),
            inputs: vec![],
            ..Default::default()
        },
        TestCase {
            name: "view/constant_width_views_row",
            outputs: Outputs::Scene(vec![(
                    include_str!("../../snapshot_tests/view/constant_width_views_row.scene.json"),
                    DEFAULT_RESOLUTION,
            )]),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "view/constant_width_views_row_with_overflow_hidden",
            outputs: Outputs::Scene(vec![(
                    include_str!("../../snapshot_tests/view/constant_width_views_row_with_overflow_hidden.scene.json"),
                    DEFAULT_RESOLUTION,
            )]),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "view/constant_width_views_row_with_overflow_visible",
            outputs: Outputs::Scene(vec![(
                    include_str!("../../snapshot_tests/view/constant_width_views_row_with_overflow_visible.scene.json"),
                    DEFAULT_RESOLUTION,
            )]),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "view/constant_width_views_row_with_overflow_fit",
            outputs: Outputs::Scene(vec![(
                    include_str!("../../snapshot_tests/view/constant_width_views_row_with_overflow_fit.scene.json"),
                    DEFAULT_RESOLUTION,
            )]),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "view/dynamic_width_views_row",
            outputs: Outputs::Scene(vec![(
                    include_str!("../../snapshot_tests/view/dynamic_width_views_row.scene.json"),
                    DEFAULT_RESOLUTION,
            )]),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "view/dynamic_and_constant_width_views_row",
            outputs: Outputs::Scene(vec![(
                    include_str!("../../snapshot_tests/view/dynamic_and_constant_width_views_row.scene.json"),
                    DEFAULT_RESOLUTION,
            )]),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "view/dynamic_and_constant_width_views_row_with_overflow",
            outputs: Outputs::Scene(vec![(
                    include_str!("../../snapshot_tests/view/dynamic_and_constant_width_views_row_with_overflow.scene.json"),
                    DEFAULT_RESOLUTION,
            )]),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "view/constant_width_and_height_views_row",
            outputs: Outputs::Scene(vec![(
                    include_str!("../../snapshot_tests/view/constant_width_and_height_views_row.scene.json"),
                    DEFAULT_RESOLUTION,
            )]),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "view/view_with_absolute_positioning_partially_covered_by_sibling",
            outputs: Outputs::Scene(vec![(
                    include_str!("../../snapshot_tests/view/view_with_absolute_positioning_partially_covered_by_sibling.scene.json"),
                    DEFAULT_RESOLUTION,
            )]),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "view/view_with_absolute_positioning_render_over_siblings",
            outputs: Outputs::Scene(vec![(
                    include_str!("../../snapshot_tests/view/view_with_absolute_positioning_render_over_siblings.scene.json"),
                    DEFAULT_RESOLUTION,
            )]),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        },
        TestCase {
            name: "view/root_view_with_background_color",
            outputs: Outputs::Scene(vec![(
                    include_str!("../../snapshot_tests/view/root_view_with_background_color.scene.json"),
                    DEFAULT_RESOLUTION,
            )]),
            inputs: vec![TestInput::new(1)],
            ..Default::default()
        }
    ])
}

fn base_snapshot_tests() -> Vec<TestCase> {
    Vec::from([TestCase {
        name: "simple_input_pass_through",
        outputs: Outputs::Scene(vec![(
            include_str!("../../snapshot_tests/simple_input_pass_through.scene.json"),
            DEFAULT_RESOLUTION,
        )]),
        inputs: vec![TestInput::new(1)],
        ..Default::default()
    }])
}
