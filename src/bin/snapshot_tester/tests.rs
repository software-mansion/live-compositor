use std::time::Duration;

use crate::utils::snapshot_test;

pub fn run_tests() {
    snapshot_test(
        "snapshot_tests/basic.json",
        vec![Duration::from_secs(2), Duration::from_secs(4)],
    );
}
