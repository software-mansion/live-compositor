use std::{fs, io};

#[path = "../../snapshot_tests/tests.rs"]
mod tests;

#[allow(dead_code)]
#[path = "../../snapshot_tests/utils.rs"]
mod utils;

#[path = "../../snapshot_tests/test_case.rs"]
mod test_case;

use tests::snapshot_tests;

fn main() {
    println!("Updating snapshots:");
    for snapshot_test in snapshot_tests() {
        if snapshot_test.run().is_ok() {
            continue;
        }

        println!("Test \"{}\"", snapshot_test.name);
        for snapshot in snapshot_test.generate_snapshots().unwrap() {
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

            image::save_buffer(
                snapshot_path,
                &snapshot.data,
                snapshot.resolution.width as u32,
                snapshot.resolution.height as u32,
                image::ColorType::Rgba8,
            )
            .unwrap();
        }
    }

    println!("Update finished");
}
