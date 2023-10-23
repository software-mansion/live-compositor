use super::test_case::{TestCase, TestInput};

pub fn snapshot_tests() -> Vec<TestCase> {
    vec![TestCase {
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
