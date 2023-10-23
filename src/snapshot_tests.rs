use std::env;

use self::tests::snapshot_tests;

mod test_case;
mod tests;
mod utils;

#[test]
fn test_snapshots() {
    if env::var("CI").is_ok() {
        return;
    }

    for snapshot_test in snapshot_tests() {
        eprintln!("Test \"{}\"", snapshot_test.name);
        if let Err(err) = snapshot_test.run() {
            panic!("{err}");
        }
    }
}
