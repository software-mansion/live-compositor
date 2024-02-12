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

use crate::common::write_example_sdp_file;

#[path = "./common/common.rs"]
mod common;

const SAMPLE_FILE_URL: &str =
    "https://sample-videos.com/video123/mp4/720/big_buck_bunny_720p_10mb.mp4";
const SAMPLE_FILE_PATH: &str = "examples/assets/big_buck_bunny_720p_10mb.mp4";
const HTML_FILE_PATH: &str = "examples/web_view.html";

const VIDEO_RESOLUTION: Resolution = Resolution {
    width: 1920,
    height: 1080,
};

fn main() {
    ffmpeg_next::format::network::init();
    logger::init_logger();

    #[cfg(feature = "web_renderer")]
    {
        use compositor_chromium::cef::bundle_for_development;

        let target_path = &std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join("..");
        if let Err(err) = bundle_for_development(target_path) {
            panic!(
                "Build process helper first. For release profile use: cargo build -r --bin process_helper. {:?}",
                err
            );
        }
    }
    thread::spawn(|| {
        if let Err(err) = start_example_client_code() {
            error!("{err}")
        }
    });
 
    http::Server::new(config().api_port).run();
}

fn start_example_client_code() -> Result<()> {
    info!("[example] Start listening on output port.");
    let output_sdp = write_example_sdp_file("127.0.0.1", 8002)?;
    Command::new("ffplay")
        .args(["-protocol_whitelist", "file,rtp,udp", &output_sdp])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    thread::sleep(Duration::from_secs(2));

    info!("[example] Download sample.");
    let sample_path = env::current_dir()?.join(SAMPLE_FILE_PATH);
    fs::create_dir_all(sample_path.parent().unwrap())?;
    common::ensure_downloaded(SAMPLE_FILE_URL, &sample_path)?;
    let file_path = env::current_dir()?
        .join(HTML_FILE_PATH)
        .display()
        .to_string();

    info!("[example] Send register input request.");
    common::post(&json!({
        "type": "register",
        "entity_type": "rtp_input_stream",
        "input_id": "input_1",
        "port": 8004,
        "video": {
            "codec": "h264"
        }
    }))?;

    info!("[example] Register web renderer transform");
    common::post(&json!({
        "type": "register",
        "entity_type": "web_renderer",
        "instance_id": "example_website",
        "url": format!("file://{file_path}"), // or other way of providing source
        "resolution": { "width": VIDEO_RESOLUTION.width, "height": VIDEO_RESOLUTION.height },
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
            "encoder_preset": "ultrafast",
            "initial": {
                "id": "embed_input_on_website",
                "type": "web_view",
                "instance_id": "example_website",
                "children": [
                    {
                        "type": "input_stream",
                        "input_id": "input_1",
                    }
                ]
            }
        }
    }))?;

    info!("[example] Start pipeline");
    common::post(&json!({
        "type": "start",
    }))?;

    info!("[example] Start input stream");
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
