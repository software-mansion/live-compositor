use integration_tests::ffmpeg::start_ffmpeg_receive;
use std::{thread, time::Duration};

fn main() {
    start_ffmpeg_receive(Some(5016), None);
    println!("started");
    thread::sleep(Duration::from_secs(300));
}
