use anyhow::Result;
use log::{error, info};
use serde_json::json;
use std::{
    env,
    process::{Command, Stdio},
    thread,
    time::Duration,
};
use video_compositor::{config::config, http, logger, types::Resolution};

use crate::common::write_video_audio_example_sdp_file;

#[path = "./common/common.rs"]
mod common;

const BUNNY_URL: &str =
    "https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4";

// const MP4_URL: &str =
//     "https://filesamples.com/samples/video/mp4/sample_960x400_ocean_with_audio.mp4";
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

    http::Server::new(config().api_port).run();
}

fn start_example_client_code() -> Result<()> {
    info!("[example] Start listening on output port.");
    let output_sdp = write_video_audio_example_sdp_file("127.0.0.1", 8002, 8004)?;
    Command::new("ffplay")
        .args(["-protocol_whitelist", "file,rtp,udp", &output_sdp])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    thread::sleep(Duration::from_secs(2));

    info!("[example] Send register input request.");
    common::post(&json!({
        "type": "register",
        "entity_type": "mp4",
        "input_id": "input_1",
        "url": BUNNY_URL
    }))?;

    let shader_source = include_str!("./silly.wgsl");
    info!("[example] Register shader transform");
    common::post(&json!({
        "type": "register",
        "entity_type": "shader",
        "shader_id": "shader_example_1",
        "source": shader_source,
    }))?;

    info!("[example] Send register output video request.");
    common::post(&json!({
        "type": "register",
        "entity_type": "output_stream",
        "output_id": "output_1",
        "port": 8002,
        "ip": "127.0.0.1",
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

    info!("[example] Send register output audio request.");
    common::post(&json!({
        "type": "register",
        "entity_type": "output_stream",
        "output_id": "output_2",
        "port": 8004,
        "ip": "127.0.0.1",
        "audio": {
            "initial": {
                "inputs": [
                    {"input_id": "input_1"}
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

    Ok(())
}
