use std::{thread, time::Duration};

use integration_tests::ffmpeg::start_ffmpeg_send;

fn main() {
    start_ffmpeg_send(
        "127.0.0.1",
        Some(5020),
        Some(5022),
        integration_tests::examples::TestSample::BigBuckBunny,
    )
    .unwrap();
    println!("started");
    thread::sleep(Duration::from_secs(300));
}
