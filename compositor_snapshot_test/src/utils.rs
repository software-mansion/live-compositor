use std::time::Duration;

use crate::test_runner::{SnapshotTestConfig, TestRunner};
use compositor_common::Frame;

pub fn snapshot_test(config_source: &str, test_name: &str, timestamps: Vec<Duration>) {
    let config: SnapshotTestConfig = serde_json::from_str(config_source).unwrap();
    let test_runner = TestRunner::new(test_name, config).unwrap();
    for pts in timestamps {
        if let Err(err) = test_runner.run(pts) {
            panic!(
                "Test \"{}\", PTS({}) failed: {}",
                test_name,
                pts.as_secs_f32(),
                err
            );
        }
    }
}

pub fn frame_to_bytes(frame: &Frame) -> Vec<u8> {
    let mut data = Vec::with_capacity(frame.resolution.width * frame.resolution.height * 3 / 2);
    data.extend_from_slice(&frame.data.y_plane);
    data.extend_from_slice(&frame.data.u_plane);
    data.extend_from_slice(&frame.data.v_plane);

    data
}

// TODO: Results may slightly differ depending on the platform. There should be an accepted margin of error here
pub fn are_snapshots_equal(old_snapshot: &[u8], new_snapshot: &[u8]) -> bool {
    old_snapshot == new_snapshot
}
