use std::{collections::HashSet, env};

use self::{
    test_case::Snapshot,
    tests::snapshot_tests,
    utils::{find_unused_snapshots, snapshots_path},
};

mod test_case;
mod tests;
mod utils;

#[test]
fn test_snapshots() {
    if env::var("CI").is_ok() {
        return;
    }

    let mut produced_snapshots = HashSet::new();
    for snapshot_test in snapshot_tests() {
        eprintln!("Test \"{}\"", snapshot_test.name);
        let snapshots = snapshot_test.generate_snapshots().unwrap();
        match snapshot_test.test_snapshots(&snapshots) {
            Ok(_) => produced_snapshots.extend(snapshots.iter().map(Snapshot::save_path)),
            Err(err) => panic!("{err}"),
        }
    }

    let unused_snapshots = find_unused_snapshots(&produced_snapshots, snapshots_path());
    if !unused_snapshots.is_empty() {
        panic!("Some snapshots were not used: {unused_snapshots:#?}")
    }
}
