use anyhow::Result;
use live_compositor::{server, types::Resolution};
use log::{error, info};
use serde_json::json;
use std::{
    env,
    thread::{self},
};

use crate::common::{download_file, start_ffplay, stream_video};

#[path = "./common/common.rs"]
mod common;

const SAMPLE_FILE_URL: &str = "https://filesamples.com/samples/video/mp4/sample_1280x720.mp4";
const SAMPLE_FILE_PATH: &str = "examples/assets/sample_1280_720.mp4";
const HTML_FILE_PATH: &str = "examples/web_view.html";

const VIDEO_RESOLUTION: Resolution = Resolution {
    width: 1920,
    height: 1080,
};

const IP: &str = "127.0.0.1";
const INPUT_PORT: u16 = 8002;
const OUTPUT_PORT: u16 = 8004;

fn main() {
    env::set_var("LIVE_COMPOSITOR_WEB_RENDERER_ENABLE", "1");
    ffmpeg_next::format::network::init();

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
    thread::spawn(|| {
        if let Err(err) = start_example_client_code() {
            error!("{err}")
        }
    });

    server::run();
}

fn start_example_client_code() -> Result<()> {
    info!("[example] Start listening on output port.");
    start_ffplay(IP, OUTPUT_PORT, None)?;

    info!("[example] Download sample.");
    let sample_path = download_file(SAMPLE_FILE_URL, SAMPLE_FILE_PATH)?;

    let html_file_path = env::current_dir()?
        .join(HTML_FILE_PATH)
        .display()
        .to_string();

    info!("[example] Send register input request.");
    common::post(
        "input/input_1/register",
        &json!({
            "type": "rtp_stream",
            "port": INPUT_PORT,
            "video": {
                "decoder": "ffmpeg_h264"
            }
        }),
    )?;

    info!("[example] Register web renderer transform");
    common::post(
        "web-renderer/example_website/register",
        &json!({
            "url": format!("file://{html_file_path}"), // or other way of providing source
            "resolution": { "width": VIDEO_RESOLUTION.width, "height": VIDEO_RESOLUTION.height },
        }),
    )?;

    info!("[example] Send register output request.");
    common::post(
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
                        "id": "embed_input_on_website",
                        "type": "web_view",
                        "instance_id": "example_website",
                        "children": [
                            {
                                "id": "big_bunny_video",
                                "type": "input_stream",
                                "input_id": "input_1",
                            }
                        ]
                    }
                }
            }
        }),
    )?;

    info!("[example] Start pipeline");
    common::post("start", &json!({}))?;

    stream_video(IP, INPUT_PORT, sample_path)?;
    Ok(())
}
