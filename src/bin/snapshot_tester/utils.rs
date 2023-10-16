use std::{
    env,
    io::{self, Write},
    path::Path,
    time::Duration,
};

use crate::test_runner::{RunMode, TestRunner};
use anyhow::Result;
use compositor_common::Frame;

pub fn snapshot_test<P: AsRef<Path>>(path: P, timestamps: Vec<Duration>) {
    let path = path.as_ref();
    let args = env::args().collect::<Vec<_>>();
    let run_mode = match args.get(1).map(String::as_str) {
        Some("interactive") => RunMode::Interactive,
        Some("update") => RunMode::UpdateSnapshots,
        Some(opt) => {
            panic!("Invalid option: {opt}\nUsage: cargo run --bin snapshot_tester -- [interactive/update]")
        }
        None => RunMode::NonInteractive,
    };

    let test_runner = TestRunner::new(run_mode, path).unwrap();
    for pts in timestamps {
        if let Err(err) = test_runner.run(pts) {
            panic!(
                "Tests for \"{}\", PTS({}) failed: {}",
                path.to_string_lossy(),
                pts.as_secs_f32(),
                err
            );
        }
    }
}

pub fn ask(question: &str) -> Result<bool> {
    print!("{question} [Y/N] ");
    io::stdout().flush()?;
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;

    let answer = match answer.to_lowercase().trim() {
        "y" => true,
        "n" => false,
        answer => {
            println!("Invalid answer \"{answer}\". Try again");
            ask(question)?
        }
    };

    Ok(answer)
}

pub fn frame_to_bytes(frame: &Frame) -> Vec<u8> {
    let mut data = Vec::with_capacity(frame.resolution.width * frame.resolution.height * 3 / 2);
    data.extend_from_slice(&frame.data.y_plane);
    data.extend_from_slice(&frame.data.u_plane);
    data.extend_from_slice(&frame.data.v_plane);

    data
}
