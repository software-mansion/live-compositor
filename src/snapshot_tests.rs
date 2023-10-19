#[path = "snapshot_tests/utils.rs"]
#[allow(dead_code)]
mod utils;

#[allow(dead_code)]
pub const SNAPSHOTS_DIR_NAME: &str = "snapshot_tests";

pub fn run_snapshot_tests() {
    // test(
    //     "basic",
    //     include_str!("../snapshot_tests/renderers_basic.json"),
    //     include_str!("../snapshot_tests/scene_basic.json"),
    //     vec![Duration::from_secs(3), Duration::from_secs(1)],
    // );
}

#[test]
fn test_snapshots() {
    run_snapshot_tests();
}
