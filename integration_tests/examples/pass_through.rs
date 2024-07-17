use anyhow::Result;
use compositor_api::types::Resolution;
use live_compositor::server;
use log::{error, info};
use serde_json::json;
use std::{env, thread, time::Duration};

use integration_tests::examples::{
    self, download_file, start_ffplay, start_websocket_thread, stream_video,
};

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
    env::set_var("LIVE_COMPOSITOR_WEB_RENDERER_ENABLE", "0");
    ffmpeg_next::format::network::init();

    thread::spawn(|| {
        if let Err(err) = start_example_client_code() {
            error!("{err}")
        }
    });

    server::run()
}

fn start_example_client_code() -> Result<()> {
    info!("[example] Start listening on output port.");
    start_ffplay(IP, OUTPUT_PORT, None)?;
    start_websocket_thread();

    info!("[example] Download sample.");
    let sample_path = download_file(SAMPLE_FILE_URL, SAMPLE_FILE_PATH)?;

    info!("[example] Send register input request.");
    examples::post(
        "input/input_1/register",
        &json!({
            "type": "rtp_stream",
            "port": INPUT_PORT,
            "video": {
                "decoder": "ffmpeg_h264"
            }
        }),
    )?;

    info!("[example] Send register output request.");
    examples::post(
        "output/output_1/register",
        &json!({
            "type": "rtp_stream",
            "ip": IP,
            "port": OUTPUT_PORT,
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
                        "id": "input_1",
                        "type": "input_stream",
                        "input_id": "input_1",
                    }
                }
            }
        }),
    )?;

    std::thread::sleep(Duration::from_millis(500));

    info!("[example] Start pipeline");
    examples::post("start", &json!({}))?;

    stream_video(IP, INPUT_PORT, sample_path)?;
    Ok(())
}
