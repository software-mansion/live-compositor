use anyhow::Result;
use log::{error, info};
use serde_json::json;
use std::{
    env, fs,
    process::{Command, Stdio},
    thread::{self},
    time::Duration,
};
use video_compositor::{config::config, http, logger, types::Resolution};

use crate::common::write_video_audio_example_sdp_file;

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

    info!("[example] Download sample.");
    let bunny_path = env::current_dir()?.join(BUNNY_FILE_PATH);
    fs::create_dir_all(bunny_path.parent().unwrap())?;
    common::ensure_downloaded(BUNNY_FILE_URL, &bunny_path)?;

    info!("[example] Download sample.");
    let sintel_path = env::current_dir()?.join(ELEPHANT_DREAM_FILE_PATH);
    fs::create_dir_all(sintel_path.parent().unwrap())?;
    common::ensure_downloaded(ELEPHANT_DREAM_FILE_URL, &sintel_path)?;

    info!("[example] Send register input request.");
    common::post(&json!({
        "type": "register",
        "entity_type": "rtp_input_stream",
        "input_id": "input_1",
        "port": 8006,
        "video": {
            "codec": "h264"
        },
    }))?;

    info!("[example] Send register input request.");
    common::post(&json!({
        "type": "register",
        "entity_type": "rtp_input_stream",
        "input_id": "input_2",
        "port": 8008,
        "audio": {
            "codec": "opus"
        },
    }))?;

    info!("[example] Send register input request.");
    common::post(&json!({
        "type": "register",
        "entity_type": "rtp_input_stream",
        "input_id": "input_3",
        "port": 8010,
        "video": {
            "codec": "h264"
        },
    }))?;

    info!("[example] Send register input request.");
    common::post(&json!({
        "type": "register",
        "entity_type": "rtp_input_stream",
        "input_id": "input_4",
        "port": 8012,
        "audio": {
            "codec": "opus"
        },
    }))?;

    info!("[example] Send register output request.");
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
        "port": 8004,
        "ip": "127.0.0.1",
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

    let path = sintel_path.clone();
    Command::new("ffmpeg")
        .args(["-stream_loop", "-1", "-re", "-i"])
        .arg(path.clone())
        .args([
            "-an",
            "-c:v",
            "copy",
            "-f",
            "rtp",
            "-bsf:v",
            "h264_mp4toannexb",
            "rtp://127.0.0.1:8006?rtcpport=8006",
        ])
        .spawn()?;

    let path = sintel_path.clone();
    Command::new("ffmpeg")
        .args(["-re", "-i"])
        .arg(path)
        .args([
            "-vn",
            "-c:a",
            "libopus",
            "-f",
            "rtp",
            "rtp://127.0.0.1:8008?rtcpport=8008",
        ])
        .spawn()?;

    let path = bunny_path.clone();
    Command::new("ffmpeg")
        .args(["-stream_loop", "-1", "-re", "-i"])
        .arg(path.clone())
        .args([
            "-an",
            "-c:v",
            "copy",
            "-f",
            "rtp",
            "-bsf:v",
            "h264_mp4toannexb",
            "rtp://127.0.0.1:8010?rtcpport=8010",
        ])
        .spawn()?;

    let path = bunny_path.clone();
    Command::new("ffmpeg")
        .args(["-re", "-i"])
        .arg(path)
        .args([
            "-vn",
            "-c:a",
            "libopus",
            "-f",
            "rtp",
            "rtp://127.0.0.1:8012?rtcpport=8012",
        ])
        .spawn()?;

    Ok(())
}
