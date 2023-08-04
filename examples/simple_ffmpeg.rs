use anyhow::Result;
use compositor_common::{scene::Resolution, Framerate};
use compositor_pipeline::Pipeline;
use log::{error, info};
use serde_json::json;
use signal_hook::{consts, iterator::Signals};
use std::{env, fs, process::Command, sync::Arc, thread, time::Duration};
use video_compositor::{http, state::State};

use crate::common::write_example_sdp_file;

#[path = "./common/common.rs"]
mod common;

const SAMPLE_FILE_URL: &str = "https://filesamples.com/samples/video/mp4/sample_1280x720.mp4";
const SAMPLE_FILE_PATH: &str = "examples/assets/sample_1280_720.mp4";
const FRAMERATE: Framerate = Framerate(30);
const VIDEO_RESOLUTION: Resolution = Resolution {
    width: 1280,
    height: 720,
};

fn main() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );
    ffmpeg_next::format::network::init();
    let pipeline = Arc::new(Pipeline::new(FRAMERATE));
    let state = Arc::new(State::new(pipeline));

    thread::spawn(|| {
        if let Err(err) = start_example_client_code() {
            error!("{err}")
        }
    });

    http::Server::new(8001, state).start();

    let mut signals = Signals::new([consts::SIGINT]).unwrap();
    signals.forever().next();
}

fn start_example_client_code() -> Result<()> {
    thread::sleep(Duration::from_secs(2));

    info!("[example] Sending init request.");
    common::post(&json!({
        "type": "init",
    }))?;

    info!("[example] Start listening on output port.");
    let output_sdp = write_example_sdp_file("127.0.0.1", 8002)?;
    Command::new("ffplay")
        .args(["-protocol_whitelist", "file,rtp,udp", &output_sdp])
        .spawn()?;

    info!("[example] Download sample.");
    let sample_path = env::current_dir()?.join(SAMPLE_FILE_PATH);
    fs::create_dir_all(sample_path.parent().unwrap())?;
    common::ensure_downloaded(SAMPLE_FILE_URL, &sample_path)?;

    info!("[example] Send register output request.");
    common::post(&json!({
        "type": "register_output",
        "id": "output 1",
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

    info!("[example] Send register input request.");
    common::post(&json!({
        "type": "register_input",
        "id": "input 1",
        "port": 8004
    }))?;

    info!("[example] Update scene");
    common::post(&json!({
        "type": "update_scene",
        "inputs": [
            {
                "input_id": "input 1",
                "resolution": { "width": VIDEO_RESOLUTION.width, "height": VIDEO_RESOLUTION.height },
            }
        ],
        "transforms": [],
        "outputs": [
            {
                "output_id": "output 1",
                "input_pad": "input 1"
            }
        ]
    }))?;

    info!("[example] Start pipeline");
    common::post(&json!({
        "type": "start",
    }))?;

    Command::new("ffmpeg")
        .args(["-re", "-i"])
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
