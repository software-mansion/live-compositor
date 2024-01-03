use anyhow::Result;
use log::{error, info};
use serde_json::json;
use std::{
    env, fs,
    process::{Command, Stdio},
    thread,
    time::Duration,
};
use video_compositor::{http, types::Resolution};

use crate::common::write_example_sdp_file;

#[path = "./common/common.rs"]
mod common;

const SAMPLE_FILE_URL: &str = "https://filesamples.com/samples/video/mp4/sample_1280x720.mp4";
const SAMPLE_FILE_PATH: &str = "examples/assets/sample_1280_720.mp4";
const VIDEO_RESOLUTION: Resolution = Resolution {
    width: 1280,
    height: 720,
};
const FRAMERATE: u32 = 30;

fn main() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );
    ffmpeg_next::format::network::init();

    thread::spawn(|| {
        if let Err(err) = start_example_client_code() {
            error!("{err}")
        }
    });

    http::Server::new(8001).run();
}

fn start_example_client_code() -> Result<()> {
    thread::sleep(Duration::from_secs(2));

    info!("[example] Sending init request.");
    common::post(&json!({
        "type": "init",
        "framerate": FRAMERATE,
        "web_renderer": {
            "init": false
        },
    }))?;

    info!("[example] Start listening on output port.");
    let output_sdp = write_example_sdp_file("127.0.0.1", 8002)?;
    Command::new("ffplay")
        .args(["-protocol_whitelist", "file,rtp,udp", &output_sdp])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    info!("[example] Download sample.");
    let sample_path = env::current_dir()?.join(SAMPLE_FILE_PATH);
    fs::create_dir_all(sample_path.parent().unwrap())?;
    common::ensure_downloaded(SAMPLE_FILE_URL, &sample_path)?;

    info!("[example] Send register output request.");
    common::post(&json!({
        "type": "register",
        "entity_type": "output_video",
        "output_id": "output_1",
        "port": 8002,
        "ip": "127.0.0.1",
        "resolution": {
            "width": VIDEO_RESOLUTION.width,
            "height": VIDEO_RESOLUTION.height,
        },
        "encoder_settings": {
            "preset": "medium"
        }
    }))?;

    info!("[example] Send register input video request.");
    common::post(&json!({
        "type": "register",
        "entity_type": "input_video",
        "input_id": "input_video_1",
        "port": 8004
    }))?;

    info!("[example] Send register input audio request.");
    common::post(&json!({
        "type": "register",
        "entity_type": "input_audio",
        "input_id": "input_audio_1",
        "port": 8006
    }))?;

    let shader_source = include_str!("./silly.wgsl");
    info!("[example] Register shader transform");
    common::post(&json!({
        "type": "register",
        "entity_type": "shader",
        "shader_id": "shader_example_1",
        "source": shader_source,
    }))?;

    info!("[example] Update composition");
    common::post(&json!({
        "type": "update_composition",
        "video_outputs": [{
            "output_id": "output_1",
            "root": {
                "type": "shader",
                "id": "shader_node_1",
                "shader_id": "shader_example_1",
                "children": [
                    {
                        "id": "input_1",
                        "type": "input_stream",
                        "input_id": "input_video_1",
                    }
                ],
                "resolution": { "width": VIDEO_RESOLUTION.width, "height": VIDEO_RESOLUTION.height },
            }
        }],
        "audio_outputs": [{
            "output_id": "output_audio_1",
            "tracks": [{
                "input_id": "input_audio_1"
            }]
        }]
    }))?;

    info!("[example] Start pipeline");
    common::post(&json!({
        "type": "start",
    }))?;

    Command::new("ffmpeg")
        .args(["-stream_loop", "-1", "-re", "-i"])
        .arg(sample_path)
        .args([
            "-an",
            "-c:v",
            "libx264",
            "-f",
            "rtp",
            "rtp://127.0.0.1:8004?rtcpport=8004",
        ])
        .spawn()?;
    Ok(())
}
