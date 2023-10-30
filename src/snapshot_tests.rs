use std::{collections::HashSet, env, fs, path::PathBuf};

use anyhow::Error;

use crate::snapshot_tests::test_case::TestCase;

use self::{
    test_case::TestCaseError,
    tests::snapshot_tests,
    utils::{find_unused_snapshots, snapshots_path},
};

mod test_case;
mod tests;
mod utils;

#[test]
fn test_snapshots() {
    let artifact_save_path = artifacts_path();
    if artifact_save_path.exists() {
        fs::remove_dir_all(artifact_save_path).unwrap();
    }

    let test_cases = snapshot_tests();
    for test_case in test_cases.iter() {
        eprintln!("Test \"{}\"", test_case.name);
        let snapshots = test_case.generate_snapshots().unwrap();
        if let Err(err) = test_case.test_snapshots(&snapshots) {
            handle_error(err);
        }
    }

    // Check for unused snapshots
    let snapshot_paths = test_cases
        .iter()
        .flat_map(TestCase::snapshot_paths)
        .collect::<HashSet<_>>();
    for path in find_unused_snapshots(&snapshot_paths, snapshots_path()) {
        println!("Removed unused snapshot {path:?}");
        fs::remove_file(path).unwrap();
    }
}

fn handle_error(err: Error) {
    let Some(test_case_err) = err.downcast_ref::<TestCaseError>() else {
        panic!("{err}");
    };
    let TestCaseError::Mismatch {
        snapshot_from_disk,
        produced_snapshot,
    } = test_case_err
    else {
        panic!("{err}");
    };

    let artifact_save_path = artifacts_path();
    if !artifact_save_path.exists() {
        fs::create_dir_all(&artifact_save_path).unwrap();
    }

    let width = produced_snapshot.resolution.width - (produced_snapshot.resolution.width % 2);
    let height = produced_snapshot.resolution.height - (produced_snapshot.resolution.height % 2);
    image::save_buffer(
        artifact_save_path.join("produced.png"),
        &produced_snapshot.data,
        width as u32,
        height as u32,
        image::ColorType::Rgba8,
    )
    .unwrap();

    snapshot_from_disk
        .save(artifact_save_path.join("original.png"))
        .unwrap();

    panic!("{err}");
}

fn artifacts_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("failed_snapshot_tests")
}
