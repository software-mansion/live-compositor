use std::time::Duration;

use super::{input::TestInput, scenes_from_json, snapshots_path, test_case::TestCase, TestRunner};

#[test]
fn tiles_transitions_tests() {
    let mut runner = TestRunner::new(snapshots_path().join("tiles_transitions"));

    let input1 = TestInput::new(1);
    let input2 = TestInput::new(2);
    let input3 = TestInput::new(3);
    let input4 = TestInput::new(4);
    let input5 = TestInput::new(5);

    runner.add(TestCase {
            name: "tiles_transitions/tile_resize_entire_component_with_parent_transition",
            scene_updates: scenes_from_json(&[
                    include_str!("../../snapshot_tests/tiles_transitions/start_tile_resize.scene.json"),
                    include_str!("../../snapshot_tests/tiles_transitions/end_tile_resize_with_view_transition.scene.json"),
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
        });

    runner.add(TestCase {
        name: "tiles_transitions/tile_resize_entire_component_without_parent_transition",
        scene_updates: scenes_from_json(&[
            include_str!("../../snapshot_tests/tiles_transitions/start_tile_resize.scene.json"),
            include_str!("../../snapshot_tests/tiles_transitions/end_tile_resize.scene.json"),
        ]),
        inputs: vec![input1.clone(), input2.clone(), input3.clone()],
        timestamps: vec![
            Duration::from_millis(0),
            Duration::from_millis(150),
            Duration::from_millis(350),
            Duration::from_millis(500),
        ],
        ..Default::default()
    });
    runner.add(TestCase {
            name: "tiles_transitions/change_order_of_3_inputs_with_id",
            scene_updates: scenes_from_json(&[
                include_str!("../../snapshot_tests/tiles_transitions/start_with_3_inputs_all_id.scene.json"),
                include_str!("../../snapshot_tests/tiles_transitions/end_with_3_inputs_3_id_different_order.scene.json"),
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
        });
    runner.add(TestCase {
        name: "tiles_transitions/replace_component_by_adding_id",
        scene_updates: scenes_from_json(&[
            include_str!(
                "../../snapshot_tests/tiles_transitions/start_with_3_inputs_no_id.scene.json"
            ),
            include_str!(
                "../../snapshot_tests/tiles_transitions/end_with_3_inputs_1_id.scene.json"
            ),
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
    });
    runner.add(TestCase {
        name: "tiles_transitions/add_2_inputs_at_the_end_to_3_tiles_scene",
        scene_updates: scenes_from_json(&[
            include_str!(
                "../../snapshot_tests/tiles_transitions/start_with_3_inputs_no_id.scene.json"
            ),
            include_str!(
                "../../snapshot_tests/tiles_transitions/end_with_5_inputs_no_id.scene.json"
            ),
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
    });
    runner.add(TestCase {
        name: "tiles_transitions/add_input_on_2nd_pos_to_3_tiles_scene",
        scene_updates: scenes_from_json(&[
            include_str!(
                "../../snapshot_tests/tiles_transitions/start_with_3_inputs_no_id.scene.json"
            ),
            include_str!(
                "../../snapshot_tests/tiles_transitions/end_with_4_inputs_1_id.scene.json"
            ),
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
    });
    runner.add(TestCase {
        name: "tiles_transitions/add_input_at_the_end_to_3_tiles_scene",
        scene_updates: scenes_from_json(&[
            include_str!(
                "../../snapshot_tests/tiles_transitions/start_with_3_inputs_no_id.scene.json"
            ),
            include_str!(
                "../../snapshot_tests/tiles_transitions/end_with_4_inputs_no_id.scene.json"
            ),
            include_str!(
                "../../snapshot_tests/tiles_transitions/after_end_with_4_inputs_no_id.scene.json"
            ),
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
    });

    runner.run()
}
