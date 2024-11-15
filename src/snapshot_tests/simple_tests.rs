use super::{input::TestInput, scene_from_json, snapshots_path, test_case::TestCase, TestRunner};

#[test]
fn simple_tests() {
    let mut runner = TestRunner::new(snapshots_path().join("simple"));

    runner.add(TestCase {
        name: "simple/simple_input_pass_through",
        scene_updates: scene_from_json(include_str!(
            "../../snapshot_tests/simple/simple_input_pass_through.scene.json"
        )),
        inputs: vec![TestInput::new(1)],
        ..Default::default()
    });

    runner.run()
}
