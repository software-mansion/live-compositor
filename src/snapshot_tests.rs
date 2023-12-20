use std::{collections::HashSet, env, fs, path::PathBuf};

use self::{
    test_case::{TestCaseError, TestCaseInstance},
    tests::snapshot_tests,
    utils::{find_unused_snapshots, snapshots_path},
};

mod test_case;
mod tests;
mod utils;

#[test]
fn test_snapshots() {
    let failed_snapshot_path = failed_snapshot_path();
    if failed_snapshot_path.exists() {
        fs::remove_dir_all(failed_snapshot_path).unwrap();
    }

    let tests: Vec<TestCaseInstance> = snapshot_tests()
        .into_iter()
        .map(TestCaseInstance::new)
        .collect();

    check_test_names_uniqueness(&tests);

    for test in tests.iter() {
        eprintln!("Test \"{}\"", test.case.name);
        if let Err(err) = test.run() {
            handle_error(err);
        }
    }

    // Check for unused snapshots
    let snapshot_paths = tests
        .iter()
        .flat_map(TestCaseInstance::snapshot_paths)
        .collect::<HashSet<_>>();
    let unused_snapshots = find_unused_snapshots(&snapshot_paths, snapshots_path());
    if !unused_snapshots.is_empty() {
        panic!("Some snapshots were not used: {unused_snapshots:#?}")
    }
}

fn handle_error(err: TestCaseError) {
    let TestCaseError::Mismatch {
        ref snapshot_from_disk,
        ref produced_snapshot,
        ..
    } = err
    else {
        panic!("{err}");
    };

    let failed_snapshot_path = failed_snapshot_path();
    if !failed_snapshot_path.exists() {
        fs::create_dir_all(&failed_snapshot_path).unwrap();
    }
    let snapshot_save_path = produced_snapshot.save_path();
    let snapshot_name = snapshot_save_path.file_name().unwrap().to_string_lossy();

    let width = produced_snapshot.resolution.width - (produced_snapshot.resolution.width % 2);
    let height = produced_snapshot.resolution.height - (produced_snapshot.resolution.height % 2);
    image::save_buffer(
        failed_snapshot_path.join(format!("mismatched_{snapshot_name}")),
        &produced_snapshot.data,
        width as u32,
        height as u32,
        image::ColorType::Rgba8,
    )
    .unwrap();

    snapshot_from_disk
        .save(failed_snapshot_path.join(format!("original_{snapshot_name}")))
        .unwrap();

    panic!("{err}");
}

fn failed_snapshot_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("failed_snapshot_tests")
}

fn check_test_names_uniqueness(tests: &[TestCaseInstance]) {
    let mut test_names = HashSet::new();
    for test in tests.iter() {
        if !test_names.insert(test.case.name) {
            panic!(
                "Multiple snapshots tests with the same name: \"{}\".",
                test.case.name
            );
        }
    }
}
