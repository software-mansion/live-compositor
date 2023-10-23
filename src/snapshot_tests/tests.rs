use super::test_case::{TestCase, TestInput};

pub fn snapshot_tests() -> Vec<TestCase> {
    vec![TestCase {
        name: "basic/test",
        inputs: vec![TestInput::new(1)],
        renderers: vec![include_str!("../../snapshot_tests/image.renderer.json")],
        scene_json: include_str!("../../snapshot_tests/basic.scene.json"),
        ..Default::default()
    }]
}
