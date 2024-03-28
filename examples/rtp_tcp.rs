use anyhow::Result;
use log::{error, info};
use serde_json::json;
use std::{process::Command, thread, time::Duration};
use video_compositor::{server, types::Resolution};

use crate::common::{download_file, start_websocket_thread};

#[path = "./common/common.rs"]
mod common;

const SAMPLE_FILE_URL: &str = "https://filesamples.com/samples/video/mp4/sample_1280x720.mp4";
const SAMPLE_FILE_PATH: &str = "examples/assets/sample_1280_720.mp4";
const VIDEO_RESOLUTION: Resolution = Resolution {
    width: 1280,
    height: 720,
};

const IP: &str = "127.0.0.1";
const INPUT_PORT: u16 = 8002;
const OUTPUT_PORT: u16 = 8004;

fn main() {
    ffmpeg_next::format::network::init();

    thread::spawn(|| {
        if let Err(err) = start_example_client_code() {
            error!("{err}")
        }
    });

    server::run()
}

fn start_example_client_code() -> Result<()> {
    info!("[example] Download sample.");
    let sample_path = download_file(SAMPLE_FILE_URL, SAMPLE_FILE_PATH)?;
    thread::sleep(Duration::from_secs(2));
    start_websocket_thread();

    info!("[example] Send register input request.");
    common::post(&json!({
        "type": "register",
        "entity_type": "rtp_input_stream",
        "transport_protocol": "tcp_server",
        "input_id": "input_1",
        "port": INPUT_PORT,
        "video": {
            "codec": "h264"
        }
    }))?;

    let shader_source = include_str!("./silly.wgsl");
    info!("[example] Register shader transform");
    common::post(&json!({
        "type": "register",
        "entity_type": "shader",
        "shader_id": "shader_example_1",
        "source": shader_source,
    }))?;

    info!("[example] Send register output request.");
    common::post(&json!({
        "type": "register",
        "entity_type": "output_stream",
        "output_id": "output_1",
        "transport_protocol": "tcp_server",
        "port": OUTPUT_PORT,
        "video": {
            "resolution": {
                "width": VIDEO_RESOLUTION.width,
                "height": VIDEO_RESOLUTION.height,
            },
            "encoder_preset": "medium",
            "initial": {
                "type": "shader",
                "id": "shader_node_1",
                "shader_id": "shader_example_1",
                "children": [
                    {
                        "id": "input_1",
                        "type": "input_stream",
                        "input_id": "input_1",
                    }
                ],
                "resolution": { "width": VIDEO_RESOLUTION.width, "height": VIDEO_RESOLUTION.height },
            }
        }
    }))?;
    let gst_output_command = format!("gst-launch-1.0 -v tcpclientsrc host={IP} port={OUTPUT_PORT} ! \"application/x-rtp-stream\" ! rtpstreamdepay ! rtph264depay ! decodebin ! videoconvert ! autovideosink");
    Command::new("bash")
        .arg("-c")
        .arg(gst_output_command)
        .spawn()?;
    std::thread::sleep(Duration::from_millis(500));

    info!("[example] Start pipeline");
    common::post(&json!({
        "type": "start",
    }))?;

    let sample_path_str = sample_path.to_string_lossy().to_string();

    let gst_input_command = [
        "gst-launch-1.0 -v ",
        "funnel name=fn ",
        &format!("filesrc location={sample_path_str} ! qtdemux ! h264parse ! rtph264pay config-interval=1 pt=96 ! .send_rtp_sink rtpsession name=session .send_rtp_src ! fn. "),
        "session.send_rtcp_src ! fn. ",
        &format!("fn. ! rtpstreampay ! tcpclientsink host={IP} port={INPUT_PORT} "),
    ].concat();

    Command::new("bash")
        .arg("-c")
        .arg(gst_input_command)
        .spawn()?;

    Ok(())
}
