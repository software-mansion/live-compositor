use anyhow::Result;
use live_compositor::{server, types::Resolution};
use log::{error, info};
use serde_json::json;
use std::{thread, time::Duration};

use integration_tests::examples::{
    self, download_file, gst_stream, start_gstplay, start_websocket_thread,
};

const SAMPLE_FILE_URL: &str =
    "https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4";
const SAMPLE_FILE_PATH: &str = "examples/assets/BigBuckBunny.mp4";
const VIDEO_RESOLUTION: Resolution = Resolution {
    width: 1280,
    height: 720,
};

const IP: &str = "127.0.0.1";
const INPUT_1_PORT: u16 = 8002;
const INPUT_2_PORT: u16 = 8004;
const OUTPUT_PORT: u16 = 8006;

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
    examples::post(
        "input/input_1/register",
        &json!({
            "type": "rtp_stream",
            "transport_protocol": "tcp_server",
            "port": INPUT_1_PORT,
            "video": {
                "decoder": "ffmpeg_h264"
            },
        }),
    )?;

    examples::post(
        "input/input_2/register",
        &json!({
            "type": "rtp_stream",
            "transport_protocol": "tcp_server",
            "port": INPUT_2_PORT,
            "audio": {
                "decoder": "opus"
            },
        }),
    )?;

    let shader_source = include_str!("./silly.wgsl");
    info!("[example] Register shader transform");
    examples::post(
        "shader/shader_example_1/register",
        &json!({
            "source": shader_source,
        }),
    )?;

    info!("[example] Send register output request.");
    examples::post(
        "output/output_1/register",
        &json!({
            "type": "rtp_stream",
            "transport_protocol": "tcp_server",
            "port": OUTPUT_PORT,
            "audio": {
                "initial": {
                    "inputs": [
                        {"input_id": "input_2"},
                    ]
                },
                "encoder": {
                   "type": "opus",
                   "channels": "stereo",
                }
            },
            "video": {
                "resolution": {
                    "width": VIDEO_RESOLUTION.width,
                    "height": VIDEO_RESOLUTION.height,
                },
                "encoder": {
                    "type": "ffmpeg_h264",
                    "preset": "ultrafast"
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

    start_gstplay(IP, OUTPUT_PORT, true, true)?;

    info!("[example] Start pipeline");
    examples::post("start", &json!({}))?;

    let sample_path_str = sample_path.to_string_lossy().to_string();

    gst_stream(IP, Some(INPUT_1_PORT), Some(INPUT_2_PORT), &sample_path_str)?;
    Ok(())
}
