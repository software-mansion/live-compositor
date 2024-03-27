use anyhow::Result;
use log::{error, info};
use serde_json::json;
use std::{
    env,
    thread::{self},
    time::Duration,
};
use video_compositor::{logger, server, types::Resolution};

use crate::common::{
    download_file, start_ffplay, start_websocket_thread, stream_audio, stream_video,
};

#[path = "./common/common.rs"]
mod common;

const BUNNY_FILE_URL: &str =
    "https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4";
const ELEPHANT_DREAM_FILE_URL: &str =
    "http://commondatastorage.googleapis.com/gtv-videos-bucket/sample/ElephantsDream.mp4";
const BUNNY_FILE_PATH: &str = "examples/assets/BigBuckBunny.mp4";
const ELEPHANT_DREAM_FILE_PATH: &str = "examples/assets/ElephantsDream.mp4";
const VIDEO_RESOLUTION: Resolution = Resolution {
    width: 1280,
    height: 720,
};

const IP: &str = "127.0.0.1";
const INPUT_1_PORT: u16 = 8002;
const INPUT_2_PORT: u16 = 8004;
const INPUT_3_PORT: u16 = 8006;
const INPUT_4_PORT: u16 = 8008;
const OUTPUT_VIDEO_PORT: u16 = 8010;
const OUTPUT_AUDIO_PORT: u16 = 8012;

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
    info!("[example] Start listening on output port.");
    start_ffplay(IP, OUTPUT_VIDEO_PORT, Some(OUTPUT_AUDIO_PORT))?;
    start_websocket_thread();

    info!("[example] Download sample.");
    let bunny_path = download_file(BUNNY_FILE_URL, BUNNY_FILE_PATH)?;

    info!("[example] Download sample.");
    let elephant_path = download_file(ELEPHANT_DREAM_FILE_URL, ELEPHANT_DREAM_FILE_PATH)?;

    info!("[example] Send register input request.");
    common::post(&json!({
        "type": "register",
        "entity_type": "rtp_input_stream",
        "input_id": "input_1",
        "port": INPUT_1_PORT,
        "video": {
            "codec": "h264"
        },
    }))?;

    info!("[example] Send register input request.");
    common::post(&json!({
        "type": "register",
        "entity_type": "rtp_input_stream",
        "input_id": "input_2",
        "port": INPUT_2_PORT,
        "audio": {
            "codec": "opus"
        },
    }))?;

    info!("[example] Send register input request.");
    common::post(&json!({
        "type": "register",
        "entity_type": "rtp_input_stream",
        "input_id": "input_3",
        "port": INPUT_3_PORT,
        "video": {
            "codec": "h264"
        },
    }))?;

    info!("[example] Send register input request.");
    common::post(&json!({
        "type": "register",
        "entity_type": "rtp_input_stream",
        "input_id": "input_4",
        "port": INPUT_4_PORT,
        "audio": {
            "codec": "opus"
        },
    }))?;

    info!("[example] Send register output request.");
    common::post(&json!({
        "type": "register",
        "entity_type": "output_stream",
        "output_id": "output_1",
        "ip": IP,
        "port": OUTPUT_VIDEO_PORT,
        "video": {
            "resolution": {
                "width": VIDEO_RESOLUTION.width,
                "height": VIDEO_RESOLUTION.height,
            },
            "encoder_preset": "medium",
            "initial": {
                "type": "tiles",
                "children": [
                    {
                        "type": "input_stream",
                        "input_id": "input_1"
                    },
                    {
                        "type": "input_stream",
                        "input_id": "input_3"
                    }
                ]
            },
            "resolution": { "width": VIDEO_RESOLUTION.width, "height": VIDEO_RESOLUTION.height },
        }
    }))?;

    info!("[example] Send register output request.");
    common::post(&json!({
        "type": "register",
        "entity_type": "output_stream",
        "output_id": "output_2",
        "ip": IP,
        "port": OUTPUT_AUDIO_PORT,
        "audio": {
            "initial": {
                "inputs": [
                    {"input_id": "input_2"},
                    {"input_id": "input_4"}
                ]
            },
            "channels": "stereo"
        }
    }))?;

    std::thread::sleep(Duration::from_millis(500));

    info!("[example] Start pipeline");
    common::post(&json!({
        "type": "start",
    }))?;

    stream_video(IP, INPUT_1_PORT, bunny_path.clone())?;
    stream_audio(IP, INPUT_2_PORT, bunny_path)?;
    stream_video(IP, INPUT_3_PORT, elephant_path.clone())?;
    stream_audio(IP, INPUT_4_PORT, elephant_path)?;

    Ok(())
}
