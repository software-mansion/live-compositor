use std::{fs, io, path::PathBuf, process::Command};

use compositor_snapshot_test::SNAPSHOTS_DIR_NAME;

fn main() {
    let saved_snapshots_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(SNAPSHOTS_DIR_NAME)
        .join("snapshots");
    if let Err(err) = fs::remove_dir_all(saved_snapshots_path) {
        if err.kind() != io::ErrorKind::NotFound {
            panic!("Failed to remove old snapshots: {err}");
        }
    }

    Command::new("cargo")
        .args(["test", "--all"])
        .output()
        .unwrap();
}
