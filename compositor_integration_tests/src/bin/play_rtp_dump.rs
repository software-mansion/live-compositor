use std::process::Command;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        eprintln!("Usage: {} <input_file>", args[0]);
        return;
    }

    let input_file = &args[1];
    let dump_packets_command = format!(
        "gst-launch-1.0 -v filesrc location={input_file} ! application/x-rtp-stream ! rtpstreamdepay ! rtph264depay ! video/x-h264,framerate=30/1 ! h264parse ! h264timestamper ! decodebin ! videoconvert ! autovideosink"
    );
    Command::new("bash")
        .arg("-c")
        .arg(dump_packets_command)
        .status()
        .unwrap();
}
