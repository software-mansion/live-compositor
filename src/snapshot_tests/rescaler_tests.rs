use compositor_render::Resolution;

use super::{
    input::TestInput, scene_from_json, snapshots_path, test_case::TestCase, TestRunner,
    DEFAULT_RESOLUTION,
};

#[test]
fn rescaler_tests() {
    let mut runner = TestRunner::new(snapshots_path().join("rescaler"));
    let default = TestCase {
        inputs: vec![TestInput::new(1)],
        ..Default::default()
    };

    let higher_than_default_resolution = Resolution {
        width: DEFAULT_RESOLUTION.width,
        height: DEFAULT_RESOLUTION.height + 100,
    };
    let lower_than_default_resolution = Resolution {
        width: DEFAULT_RESOLUTION.width,
        height: DEFAULT_RESOLUTION.height - 100,
    };
    let portrait_resolution = Resolution {
        width: 360,
        height: 640,
    };
    let higher_than_default = TestInput::new_with_resolution(1, higher_than_default_resolution);
    let lower_than_default = TestInput::new_with_resolution(1, lower_than_default_resolution);
    let portrait = TestInput::new_with_resolution(1, portrait_resolution);

    runner.add(TestCase {
        name: "rescaler/fit_view_with_known_height",
        scene_updates: scene_from_json(include_str!(
            "../../snapshot_tests/rescaler/fit_view_with_known_height.scene.json"
        )),
        ..default.clone()
    });
    runner.add(TestCase {
        name: "rescaler/fit_view_with_known_width",
        scene_updates: scene_from_json(include_str!(
            "../../snapshot_tests/rescaler/fit_view_with_known_width.scene.json"
        )),
        ..default.clone()
    });
    runner.add(TestCase {
        name: "rescaler/fit_view_with_unknown_width_and_height",
        scene_updates: scene_from_json(include_str!(
            "../../snapshot_tests/rescaler/fit_view_with_unknown_width_and_height.scene.json"
        )),
        ..default.clone()
    });
    runner.add(TestCase {
        name: "rescaler/fill_input_stream_inverted_aspect_ratio_align_top_left",
        scene_updates: scene_from_json(include_str!(
            "../../snapshot_tests/rescaler/fill_input_stream_align_top_left.scene.json"
        )),
        inputs: vec![portrait.clone()],
        ..default.clone()
    });
    runner.add(TestCase {
        name: "rescaler/fill_input_stream_inverted_aspect_ratio_align_bottom_right",
        scene_updates: scene_from_json(include_str!(
            "../../snapshot_tests/rescaler/fill_input_stream_align_bottom_right.scene.json"
        )),
        inputs: vec![portrait.clone()],
        ..default.clone()
    });
    runner.add(TestCase {
        name: "rescaler/fill_input_stream_lower_aspect_ratio_align_bottom_right",
        scene_updates: scene_from_json(include_str!(
            "../../snapshot_tests/rescaler/fill_input_stream_align_bottom_right.scene.json"
        )),
        inputs: vec![lower_than_default.clone()],
        ..default.clone()
    });
    runner.add(TestCase {
        name: "rescaler/fill_input_stream_lower_aspect_ratio",
        scene_updates: scene_from_json(include_str!(
            "../../snapshot_tests/rescaler/fill_input_stream.scene.json"
        )),
        inputs: vec![lower_than_default.clone()],
        ..default.clone()
    });
    runner.add(TestCase {
        name: "rescaler/fill_input_stream_higher_aspect_ratio",
        scene_updates: scene_from_json(include_str!(
            "../../snapshot_tests/rescaler/fill_input_stream.scene.json"
        )),
        inputs: vec![higher_than_default.clone()],
        ..default.clone()
    });
    runner.add(TestCase {
        name: "rescaler/fill_input_stream_inverted_aspect_ratio",
        scene_updates: scene_from_json(include_str!(
            "../../snapshot_tests/rescaler/fill_input_stream.scene.json"
        )),
        inputs: vec![portrait.clone()],
        ..default.clone()
    });
    runner.add(TestCase {
        name: "rescaler/fill_input_stream_matching_aspect_ratio",
        scene_updates: scene_from_json(include_str!(
            "../../snapshot_tests/rescaler/fill_input_stream.scene.json"
        )),
        inputs: vec![TestInput::new(1)],
        ..default.clone()
    });
    runner.add(TestCase {
        name: "rescaler/fit_input_stream_lower_aspect_ratio",
        scene_updates: scene_from_json(include_str!(
            "../../snapshot_tests/rescaler/fit_input_stream.scene.json"
        )),
        inputs: vec![lower_than_default.clone()],
        ..default.clone()
    });
    runner.add(TestCase {
        name: "rescaler/fit_input_stream_higher_aspect_ratio",
        scene_updates: scene_from_json(include_str!(
            "../../snapshot_tests/rescaler/fit_input_stream.scene.json"
        )),
        inputs: vec![higher_than_default.clone()],
        ..default.clone()
    });
    runner.add(TestCase {
        name: "rescaler/fit_input_stream_higher_aspect_ratio_small_resolution",
        scene_updates: scene_from_json(include_str!(
            "../../snapshot_tests/rescaler/fit_input_stream.scene.json"
        )),
        inputs: vec![TestInput::new_with_resolution(
            1,
            Resolution {
                width: higher_than_default_resolution.width / 10,
                height: higher_than_default_resolution.height / 10,
            },
        )],
        ..default.clone()
    });
    runner.add(TestCase {
        name: "rescaler/fit_input_stream_inverted_aspect_ratio_align_top_left",
        scene_updates: scene_from_json(include_str!(
            "../../snapshot_tests/rescaler/fit_input_stream_align_top_left.scene.json"
        )),
        inputs: vec![portrait.clone()],
        ..default.clone()
    });
    runner.add(TestCase {
        name: "rescaler/fit_input_stream_inverted_aspect_ratio_align_bottom_right",
        scene_updates: scene_from_json(include_str!(
            "../../snapshot_tests/rescaler/fit_input_stream_align_bottom_right.scene.json"
        )),
        inputs: vec![portrait.clone()],
        ..default.clone()
    });
    runner.add(TestCase {
        name: "rescaler/fit_input_stream_lower_aspect_ratio_align_bottom_right",
        scene_updates: scene_from_json(include_str!(
            "../../snapshot_tests/rescaler/fit_input_stream_align_bottom_right.scene.json"
        )),
        inputs: vec![lower_than_default.clone()],
        ..default.clone()
    });
    runner.add(TestCase {
        name: "rescaler/fit_input_stream_inverted_aspect_ratio",
        scene_updates: scene_from_json(include_str!(
            "../../snapshot_tests/rescaler/fit_input_stream.scene.json"
        )),
        inputs: vec![portrait.clone()],
        ..default.clone()
    });
    runner.add(TestCase {
        name: "rescaler/fit_input_stream_matching_aspect_ratio",
        scene_updates: scene_from_json(include_str!(
            "../../snapshot_tests/rescaler/fit_input_stream.scene.json"
        )),
        ..default.clone()
    });
    runner.add(TestCase {
        name: "rescaler/border_radius",
        scene_updates: scene_from_json(include_str!(
            "../../snapshot_tests/rescaler/border_radius.scene.json"
        )),
        ..default.clone()
    });
    runner.add(TestCase {
        name: "rescaler/border_width",
        scene_updates: scene_from_json(include_str!(
            "../../snapshot_tests/rescaler/border_width.scene.json"
        )),
        ..default.clone()
    });
    runner.add(TestCase {
        name: "rescaler/box_shadow",
        scene_updates: scene_from_json(include_str!(
            "../../snapshot_tests/rescaler/box_shadow.scene.json"
        )),
        ..default.clone()
    });
    runner.add(TestCase {
        name: "rescaler/border_radius_border_box_shadow",
        scene_updates: scene_from_json(include_str!(
            "../../snapshot_tests/rescaler/border_radius_border_box_shadow.scene.json"
        )),
        ..default.clone()
    });
    runner.add(TestCase {
        name: "rescaler/border_radius_box_shadow",
        scene_updates: scene_from_json(include_str!(
            "../../snapshot_tests/rescaler/border_radius_box_shadow.scene.json"
        )),
        ..default.clone()
    });
    runner.add(TestCase {
        name: "rescaler/border_radius_box_shadow_fit_input_stream",
        scene_updates: scene_from_json(include_str!(
            "../../snapshot_tests/rescaler/border_radius_box_shadow_fit_input_stream.scene.json"
        )),
        ..default.clone()
    });
    runner.add(TestCase {
        name: "rescaler/border_radius_box_shadow_fill_input_stream",
        scene_updates: scene_from_json(include_str!(
            "../../snapshot_tests/rescaler/border_radius_box_shadow_fill_input_stream.scene.json"
        )),
        ..default.clone()
    });
    runner.add(TestCase {
        name: "rescaler/nested_border_width_radius",
        scene_updates: scene_from_json(include_str!(
            "../../snapshot_tests/rescaler/nested_border_width_radius.scene.json"
        )),
        ..default.clone()
    });
    runner.add(TestCase {
        name: "rescaler/nested_border_width_radius_aligned",
        scene_updates: scene_from_json(include_str!(
            "../../snapshot_tests/rescaler/nested_border_width_radius_aligned.scene.json"
        )),
        ..default.clone()
    });
    runner.add(TestCase {
        // it is supposed to be cut off because of the rescaler that wraps it
        name: "rescaler/border_radius_border_box_shadow_rescaled",
        scene_updates: scene_from_json(include_str!(
            "../../snapshot_tests/rescaler/border_radius_border_box_shadow_rescaled.scene.json"
        )),
        ..default.clone()
    });
    runner.run()
}
