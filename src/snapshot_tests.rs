use std::env;

use self::tests::snapshot_test_runners;

mod tests;
mod utils;

#[test]
fn test_snapshots() {
    if env::var("CI").is_ok() {
        return;
    }

    for scene_test in snapshot_test_runners() {
        if let Err(err) = scene_test.run() {
            panic!("{err}");
        }
    }
}
