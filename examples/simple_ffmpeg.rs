use anyhow::Result;
use compositor_pipeline::Pipeline;
use log::{error, info};
use serde_json::json;
use signal_hook::{consts, iterator::Signals};
use std::{env, fs, process::Command, sync::Arc, thread, time::Duration};
use video_compositor::{http, state::State};

use crate::common::write_example_sdp_file;

#[path = "./common/common.rs"]
mod common;

fn main() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );
    ffmpeg_next::format::network::init();
    let pipeline = Arc::new(Pipeline::new());
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
    let output_sdp = write_example_sdp_file(8002)?;
    Command::new("ffplay")
        .args(["-protocol_whitelist", "file,rtp,udp", &output_sdp])
        .spawn()?;

    info!("[example] Download sample.");
    let sample_path = env::current_dir()?.join("examples/assets/sample_1280_720.mp4");
    fs::create_dir_all(sample_path.parent().unwrap())?;
    common::ensure_downloaded(
        "https://filesamples.com/samples/video/mp4/sample_1280x720.mp4",
        &sample_path,
    )?;

    info!("[example] Send register output request.");
    common::post(&json!({
        "type": "register_output",
        "port": 8002,
        "resolution": {
            "width": 1280,
            "height": 720,
        },
        "encoder_settings": {
            "preset": "medium"
        }
    }))?;

    info!("[example] Send register input request.");
    common::post(&json!({
        "type": "register_input",
        "port": 8004
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
            "rtp://127.0.0.1:8004",
        ])
        .spawn()?;
    Ok(())
}
