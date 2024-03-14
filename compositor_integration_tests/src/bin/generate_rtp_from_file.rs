use std::{path::PathBuf, process::Command};

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        eprintln!("Usage: {} <input_file>", args[0]);
        return;
    }

    let input_path = PathBuf::from(&args[1]);
    let output_path = input_path
        .parent()
        .unwrap()
        .join(input_path.file_stem().unwrap());

    let input_path = input_path.to_str().unwrap();
    let output_path = output_path.to_str().unwrap();

    let dump_packets_command = format!(
        "gst-launch-1.0 -v funnel name=fn filesrc location={input_path} ! qtdemux ! h264parse ! rtph264pay config-interval=1 pt=96  ! .send_rtp_sink rtpsession name=session .send_rtp_src ! fn. session.send_rtcp_src ! fn. fn. ! rtpstreampay ! filesink location={output_path}.rtp"
    );
    Command::new("bash")
        .arg("-c")
        .arg(dump_packets_command)
        .status()
        .unwrap();
}
