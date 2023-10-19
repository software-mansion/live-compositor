use std::time::Duration;

#[path = "snapshot_tests/utils.rs"]
#[allow(dead_code)]
mod utils;

use utils::snapshot_test;

use self::utils::SceneTest;

#[allow(dead_code)]
pub const SNAPSHOTS_DIR_NAME: &str = "snapshot_tests";

pub fn snapshot_test_runners() -> Vec<SceneTest> {
    vec![snapshot_test(
        "basic",
        vec![include_str!("../snapshot_tests/image_renderer.json")],
        include_str!("../snapshot_tests/basic_scene.json"),
        vec![Duration::from_secs(3), Duration::from_secs(1)],
    )]
}

#[test]
fn test_snapshots() {
    for scene_test in snapshot_test_runners() {
        if let Err(err) = scene_test.run() {
            panic!("{err}");
        }
    }
}
