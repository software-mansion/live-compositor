use std::time::Duration;

use super::{scenes_from_json, snapshots_path, test_case::TestCase, TestRunner};

#[test]
fn transitions_tests() {
    let mut runner = TestRunner::new(snapshots_path().join("transition"));
    let default = TestCase {
        timestamps: vec![
            Duration::from_millis(0),
            Duration::from_millis(2500),
            Duration::from_millis(5000),
            Duration::from_millis(7500),
            Duration::from_millis(9000),
            Duration::from_millis(10000),
        ],
        ..Default::default()
    };

    runner.add(TestCase {
        name: "transition/change_rescaler_absolute_and_send_next_update",
        scene_updates: scenes_from_json(&[
            include_str!(
                "../../snapshot_tests/transition/change_rescaler_absolute_start.scene.json"
            ),
            include_str!("../../snapshot_tests/transition/change_rescaler_absolute_end.scene.json"),
            include_str!(
                "../../snapshot_tests/transition/change_rescaler_absolute_after_end.scene.json"
            ),
        ]),
        ..default.clone()
    });
    runner.add(TestCase {
        name: "transition/change_view_width_and_send_abort_transition",
        scene_updates: scenes_from_json(&[
            include_str!("../../snapshot_tests/transition/change_view_width_start.scene.json"),
            include_str!("../../snapshot_tests/transition/change_view_width_end.scene.json"),
            include_str!(
                "../../snapshot_tests/transition/change_view_width_after_end_without_id.scene.json"
            ),
        ]),
        ..default.clone()
    });
    runner.add(TestCase {
        name: "transition/change_view_width_and_send_next_update",
        scene_updates: scenes_from_json(&[
            include_str!("../../snapshot_tests/transition/change_view_width_start.scene.json"),
            include_str!("../../snapshot_tests/transition/change_view_width_end.scene.json"),
            include_str!("../../snapshot_tests/transition/change_view_width_after_end.scene.json"),
        ]),
        ..default.clone()
    });
    runner.add(TestCase {
        name: "transition/change_view_width",
        scene_updates: scenes_from_json(&[
            include_str!("../../snapshot_tests/transition/change_view_width_start.scene.json"),
            include_str!("../../snapshot_tests/transition/change_view_width_end.scene.json"),
        ]),
        ..default.clone()
    });
    runner.add(TestCase {
        name: "transition/change_view_height",
        scene_updates: scenes_from_json(&[
            include_str!("../../snapshot_tests/transition/change_view_height_start.scene.json"),
            include_str!("../../snapshot_tests/transition/change_view_height_end.scene.json"),
        ]),
        ..default.clone()
    });
    runner.add(TestCase {
        name: "transition/change_view_absolute",
        scene_updates: scenes_from_json(&[
            include_str!("../../snapshot_tests/transition/change_view_absolute_start.scene.json"),
            include_str!("../../snapshot_tests/transition/change_view_absolute_end.scene.json"),
        ]),
        ..default.clone()
    });
    runner.add(TestCase {
        name: "transition/change_view_absolute_cubic_bezier",
        scene_updates: scenes_from_json(&[
            include_str!("../../snapshot_tests/transition/change_view_absolute_cubic_bezier_start.scene.json"),
            include_str!("../../snapshot_tests/transition/change_view_absolute_cubic_bezier_end.scene.json"),
        ]),
        ..default.clone()
    });
    runner.add(TestCase {
        name: "transition/change_view_absolute_cubic_bezier_linear_like",
        scene_updates: scenes_from_json(&[
            include_str!("../../snapshot_tests/transition/change_view_absolute_cubic_bezier_linear_like_start.scene.json"),
            include_str!("../../snapshot_tests/transition/change_view_absolute_cubic_bezier_linear_like_end.scene.json"),
        ]),
        ..default.clone()
    });

    runner.run()
}
