use std::{fs, path::PathBuf};

#[path = "../../snapshot_tests/tests.rs"]
mod tests;
#[allow(dead_code)]
#[path = "../../snapshot_tests/utils.rs"]
mod utils;

use tests::snapshot_test_runners;
use utils::SNAPSHOTS_DIR_NAME;

fn main() {
    let saved_snapshots_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(SNAPSHOTS_DIR_NAME)
        .join("snapshots");

    for snapshot_file in fs::read_dir(saved_snapshots_path).unwrap() {
        let snapshot_file = snapshot_file.unwrap();
        if snapshot_file.file_name() == ".git" {
            continue;
        }
        fs::remove_file(snapshot_file.path()).unwrap();
    }

    println!("Updating all snapshots:");
    for test_runner in snapshot_test_runners() {
        for snapshot in test_runner.generate_snapshots().unwrap() {
            fs::write(snapshot.save_path(), snapshot.data).unwrap();
        }
    }
}
