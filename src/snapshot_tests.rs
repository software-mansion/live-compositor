use std::env;

use self::tests::snapshot_tests;

mod tests;
mod utils;

#[test]
fn test_snapshots() {
    if env::var("CI").is_ok() {
        return;
    }

    for snapshot_test in snapshot_tests() {
        eprintln!("Test \"{}\"", snapshot_test.name);
        let scene_test = snapshot_test.into_scene_test();
        if let Err(err) = scene_test.run() {
            panic!("{err}");
        }
    }
}
