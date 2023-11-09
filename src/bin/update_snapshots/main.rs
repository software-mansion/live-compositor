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
    test_case::TestCaseInstance,
    utils::{find_unused_snapshots, snapshots_path},
};

fn main() {
    println!("Updating snapshots:");

    let test: Vec<_> = snapshot_tests()
        .into_iter()
        .map(TestCaseInstance::new)
        .collect();
    for test in test.iter() {
        for pts in &test.case.timestamps {
            let (snapshots, Err(_)) = test.test_snapshots_for_pts(*pts) else {
                println!("PASS: \"{}\" (pts: {}ms)", test.case.name, pts.as_millis());
                continue;
            };

            println!(
                "UPDATE: \"{}\" (pts: {}ms)",
                test.case.name,
                pts.as_millis()
            );

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
    }

    // Check for unused snapshots
    let snapshot_paths = test
        .iter()
        .flat_map(TestCaseInstance::snapshot_paths)
        .collect::<HashSet<_>>();
    for path in find_unused_snapshots(&snapshot_paths, snapshots_path()) {
        println!("Removed unused snapshot {path:?}");
        fs::remove_file(path).unwrap();
    }

    println!("Update finished");
}
