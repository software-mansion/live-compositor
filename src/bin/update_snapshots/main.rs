use std::{collections::HashSet, fs, io};

#[path = "../../snapshot_tests/tests.rs"]
mod tests;

#[allow(dead_code)]
#[path = "../../snapshot_tests/utils.rs"]
mod utils;

#[path = "../../snapshot_tests/test_case.rs"]
mod test_case;

use tests::snapshot_tests;

use crate::{
    test_case::TestCase,
    utils::{find_unused_snapshots, snapshots_path},
};

fn main() {
    println!("Updating snapshots:");

    let test_cases = snapshot_tests();
    for snapshot_test in test_cases.iter() {
        let snapshots = snapshot_test.generate_snapshots().unwrap();
        let was_test_successful = snapshot_test.test_snapshots(&snapshots).is_ok();
        if was_test_successful {
            println!("PASS: \"{}\"", snapshot_test.name);
            continue;
        } else {
            println!("UPDATE: \"{}\"", snapshot_test.name);
        }

        for snapshot in snapshots {
            let snapshot_path = snapshot.save_path();

            if let Err(err) = fs::remove_file(&snapshot_path) {
                if err.kind() != io::ErrorKind::NotFound {
                    panic!("Failed to remove old snapshots: {err}");
                }
            }
            let parent_folder = snapshot_path.parent().unwrap();
            if !parent_folder.exists() {
                fs::create_dir_all(parent_folder).unwrap();
            }

            let width = snapshot.resolution.width - (snapshot.resolution.width % 2);
            let height = snapshot.resolution.height - (snapshot.resolution.height % 2);
            image::save_buffer(
                snapshot_path,
                &snapshot.data,
                width as u32,
                height as u32,
                image::ColorType::Rgba8,
            )
            .unwrap();
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

    println!("Update finished");
}
