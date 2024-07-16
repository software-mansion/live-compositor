use anyhow::Result;
use live_compositor::{server, types::Resolution};
use log::{error, info};
use serde_json::json;
use std::{env, process::Command, thread, time::Duration};

use integration_tests::utils::{self, start_websocket_thread};

const HLS_URL: &str = "https://raw.githubusercontent.com/membraneframework/membrane_http_adaptive_stream_plugin/master/test/membrane_http_adaptive_stream/integration_test/fixtures/audio_multiple_video_tracks/index.m3u8";
const VIDEO_RESOLUTION: Resolution = Resolution {
    width: 1280,
    height: 720,
};

const IP: &str = "127.0.0.1";
const INPUT_PORT: u16 = 8002;
const OUTPUT_PORT: u16 = 8004;

fn main() {
    env::set_var("LIVE_COMPOSITOR_WEB_RENDERER_ENABLE", "0");
    ffmpeg_next::format::network::init();

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
    utils::post(
        "input/input_1/register",
        &json!({
            "type": "rtp_stream",
            "transport_protocol": "tcp_server",
            "port": INPUT_PORT,
            "video": {
                "decoder": "ffmpeg_h264"
            }
        }),
    )?;

    let shader_source = include_str!("./silly.wgsl");
    info!("[example] Register shader transform");
    utils::post(
        "shader/shader_example_1/register",
        &json!({
            "source": shader_source,
        }),
    )?;

    info!("[example] Send register output request.");
    utils::post(
        "output/output_1/register",
        &json!({
            "type": "rtp_stream",
            "transport_protocol": "tcp_server",
            "port": OUTPUT_PORT,
            "video": {
                "resolution": {
                    "width": VIDEO_RESOLUTION.width,
                    "height": VIDEO_RESOLUTION.height,
                },
                "encoder": {
                    "type": "ffmpeg_h264",
                    "preset": "fast",
                },
                "initial": {
                    "root": {
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
            }
        }),
    )?;

    let gst_output_command = format!("gst-launch-1.0 -v tcpclientsrc host={IP} port={OUTPUT_PORT} ! \"application/x-rtp-stream\" ! rtpstreamdepay ! rtph264depay ! decodebin ! videoconvert ! autovideosink");
    Command::new("bash")
        .arg("-c")
        .arg(gst_output_command)
        .spawn()?;
    std::thread::sleep(Duration::from_millis(500));

    info!("[example] Start pipeline");
    utils::post("start", &json!({}))?;

    let gst_input_command = format!("gst-launch-1.0 -v souphttpsrc location={HLS_URL} ! hlsdemux ! qtdemux ! h264parse ! rtph264pay config-interval=1 pt=96 ! .send_rtp_sink rtpsession .send_rtp_src ! rtpstreampay ! tcpclientsink host=127.0.0.1 port={INPUT_PORT}");
    Command::new("bash")
        .arg("-c")
        .arg(gst_input_command)
        .spawn()?;

    Ok(())
}
