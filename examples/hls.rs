use anyhow::Result;
use log::{error, info};
use serde_json::json;
use std::{env, process::Command, thread, time::Duration};
use video_compositor::{logger, server, types::Resolution};

use crate::common::start_websocket_thread;

#[path = "./common/common.rs"]
mod common;

const HLS_URL: &str = "https://raw.githubusercontent.com/membraneframework/membrane_http_adaptive_stream_plugin/master/test/membrane_http_adaptive_stream/integration_test/fixtures/audio_multiple_video_tracks/index.m3u8";
const VIDEO_RESOLUTION: Resolution = Resolution {
    width: 1280,
    height: 720,
};

fn main() {
    env::set_var("LIVE_COMPOSITOR_WEB_RENDERER_ENABLE", "0");
    ffmpeg_next::format::network::init();
    logger::init_logger();

    thread::spawn(|| {
        if let Err(err) = start_example_client_code() {
            error!("{err}")
        }
    });

    server::run();
}

fn start_example_client_code() -> Result<()> {
    thread::sleep(Duration::from_secs(2));
    start_websocket_thread();

    info!("[example] Send register input request.");
    common::post(&json!({
        "type": "register",
        "entity_type": "rtp_input_stream",
        "transport_protocol": "tcp_server",
        "input_id": "input_1",
        "port": 8004,
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
        "port": 8002,
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
    let gst_output_command = "gst-launch-1.0 -v tcpclientsrc host=127.0.0.1 port=8002 ! \"application/x-rtp-stream\" ! rtpstreamdepay ! rtph264depay ! decodebin ! videoconvert ! autovideosink".to_string();
    Command::new("bash")
        .arg("-c")
        .arg(gst_output_command)
        .spawn()?;
    std::thread::sleep(Duration::from_millis(500));

    info!("[example] Start pipeline");
    common::post(&json!({
        "type": "start",
    }))?;

    let gst_input_command = format!("gst-launch-1.0 -v souphttpsrc location={HLS_URL} ! hlsdemux ! qtdemux ! h264parse ! rtph264pay config-interval=1 pt=96 ! .send_rtp_sink rtpsession .send_rtp_src ! rtpstreampay ! tcpclientsink host=127.0.0.1 port=8004");
    Command::new("bash")
        .arg("-c")
        .arg(gst_input_command)
        .spawn()?;

    Ok(())
}
