use std::{fs, io, path::PathBuf};

#[path = "../../snapshot_tests/tests.rs"]
mod tests;
#[allow(dead_code)]
#[path = "../../snapshot_tests/utils.rs"]
mod utils;

use tests::snapshot_test_runners;
use utils::SNAPSHOTS_DIR_NAME;

fn main() {
    let saved_snapshots_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(SNAPSHOTS_DIR_NAME);
    if let Err(err) = fs::remove_dir_all(&saved_snapshots_path) {
        if err.kind() != io::ErrorKind::NotFound {
            panic!("Failed to remove old snapshots: {err}");
        }
    }

    fs::create_dir(saved_snapshots_path).unwrap();

    println!("Updating all snapshots:");
    for test_runner in snapshot_test_runners() {
        for snapshot in test_runner.generate_snapshots().unwrap() {
            image::save_buffer(
                snapshot.save_path(),
                &snapshot.data,
                snapshot.resolution.width as u32,
                snapshot.resolution.height as u32,
                image::ColorType::Rgba8,
            )
            .unwrap();
        }
    }
}
