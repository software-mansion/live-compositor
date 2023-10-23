use super::utils::{SnapshotTest, TestInput};

pub fn snapshot_tests() -> Vec<SnapshotTest> {
    vec![SnapshotTest {
        name: "basic/test",
        inputs: vec![TestInput {
            name: "input1",
            ..Default::default()
        }],
        register_renderer_jsons: vec![include_str!("../../snapshot_tests/image_renderer.json")],
        scene_json: include_str!("../../snapshot_tests/basic_scene.json"),
        ..Default::default()
    }]
}
