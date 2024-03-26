use anyhow::Result;
use log::{error, info};
use serde_json::json;
use std::{env, thread, time::Duration};
use video_compositor::{logger, server, types::Resolution};

use crate::common::{download_file, start_ffplay, start_websocket_thread, stream_video};

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
    env::set_var("LIVE_COMPOSITOR_WEB_RENDERER_ENABLE", "0");
    ffmpeg_next::format::network::init();
    logger::init_logger();

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
    common::post(&json!({
        "type": "register",
        "entity_type": "rtp_input_stream",
        "input_id": "input_1",
        "port": INPUT_PORT,
        "video": {
            "codec": "h264"
        }
    }))?;

    info!("[example] Send register output request.");
    common::post(&json!({
        "type": "register",
        "entity_type": "output_stream",
        "output_id": "output_1",
        "ip": IP,
        "port": OUTPUT_PORT,
        "video": {
            "resolution": {
                "width": VIDEO_RESOLUTION.width,
                "height": VIDEO_RESOLUTION.height,
            },
            "encoder_preset": "medium",
            "initial": {
                "id": "input_1",
                "type": "input_stream",
                "input_id": "input_1",
            }
        }
    }))?;

    std::thread::sleep(Duration::from_millis(500));

    info!("[example] Start pipeline");
    common::post(&json!({
        "type": "start",
    }))?;

    stream_video(IP, OUTPUT_PORT, sample_path)?;
    Ok(())
}
