use std::time::Duration;

use super::utils::{snapshot_test, SceneTest};

pub fn snapshot_test_runners() -> Vec<SceneTest> {
    vec![snapshot_test(
        "basic",
        vec![include_str!("../../snapshot_tests/image_renderer.json")],
        include_str!("../../snapshot_tests/basic_scene.json"),
        vec![Duration::from_secs(3), Duration::from_secs(1)],
    )]
}
