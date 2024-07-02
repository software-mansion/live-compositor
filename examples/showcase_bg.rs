use anyhow::Result;
use common::download_file;
use live_compositor::{server, types::Resolution};
use log::{error, info};
use serde_json::json;
use std::{
    thread::{self},
    time::Duration,
};

use crate::common::{start_ffplay, start_websocket_thread};

#[path = "./common/common.rs"]
mod common;

const BUNNY_URL: &str =
    "https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4";
const TV_PATH: &str = "./examples/assets/tv.mp4";
const BG_PATH: &str = "./examples/assets/news_room.jpeg";

const TV_BACKGROUND_URL: &str =
  "https://raw.githubusercontent.com/membraneframework-labs/video_compositor_snapshot_tests/main/demo_assets/news_room.jpeg";

const VIDEO_RESOLUTION: Resolution = Resolution {
    width: 1920,
    height: 1080,
};

const IP: &str = "127.0.0.1";
const OUTPUT_VIDEO_PORT: u16 = 8002;
const OUTPUT_AUDIO_PORT: u16 = 8004;

fn main() {
    ffmpeg_next::format::network::init();

    thread::spawn(|| {
        if let Err(err) = start_example_client_code() {
            error!("{err}")
        }
    });

    server::run();
}

fn start_example_client_code() -> Result<()> {
    info!("[example] Start listening on output port.");
    start_ffplay(IP, OUTPUT_VIDEO_PORT, Some(OUTPUT_VIDEO_PORT))?;
    start_websocket_thread();

    const BUNNY_PATH: &str = "./examples/assets/bunny.mp4";
    download_file(BUNNY_URL, BUNNY_PATH)?;

    info!("[example] Send register input request.");
    common::post(
        "input/bunny/register",
        &json!({
            "type": "mp4",
            "path": BUNNY_PATH
        }),
    )?;

    info!("[example] Send register input request.");
    common::post(
        "input/tv/register",
        &json!({
            "type": "mp4",
            "path": TV_PATH
        }),
    )?;

    info!("[example] Send register shader request.");
    common::post(
        "shader/showcase/register",
        &json!({
            "source": include_str!("./showcase.wgsl")
        }),
    )?;

    const TV_BACKGROUND_PATH: &str = "./examples/assets/news_room.jpeg";
    download_file(TV_BACKGROUND_URL, TV_BACKGROUND_PATH)?;

    info!("[example] Send register image request.");
    common::post(
        "image/tv_background/register",
        &json!({
            "path": TV_BACKGROUND_PATH,
            "asset_type": "jpeg"
        }),
    )?;

    info!("[example] Send register image request.");
    common::post(
        "image/background/register",
        &json!({
            "path": BG_PATH,
            "asset_type": "jpeg"
        }),
    )?;

    info!("[example] Send register output video request.");
    common::post(
        "output/output_video/register",
        &json!({
            "type": "rtp_stream",
            "ip": IP,
            "port": OUTPUT_VIDEO_PORT,
            "video": {
                "resolution": {
                    "width": VIDEO_RESOLUTION.width,
                    "height": VIDEO_RESOLUTION.height,
                },
                "encoder": {
                    "type": "ffmpeg_h264",
                    "preset": "slower"
                },
                "initial": {
                    "root": {
                       "type": "view",
                       "children": [
                        {
                            "type": "view",
                            "children": [
                                {
                                    "type": "image",
                                    "image_id": "background",
                                },
                            ]
                        },
                        {
                            "type": "view",
                            "height": 120,
                            "left": 0,
                            "bottom": 0,
                            "background_color_rgba": "#FFFF00FF",
                            "children": [
                                { "type": "view" },
                                {
                                    "type": "text",
                                    "text": "LiveCompositor 😃😍",
                                    "font_size": 100,
                                    "weight": "bold",
                                    "color_rgba": "#000000FF",
                                },
                                { "type": "view" }
                            ],
                          },
                       ]
                       
                    }
                }
            }
        }),
    )?;

    // Only to record mp4
    info!("[example] Send register output audio request.");
    common::post(
        "output/output_audio/register",
        &json!({
            "type": "rtp_stream",
            "ip": IP,
            "port": OUTPUT_AUDIO_PORT,
            "audio": {
                "encoder": {
                    "type": "opus",
                    "channels": "stereo"
                },
                "initial": {
                    "inputs": [
                        {"input_id": "tv"},
                        {"input_id": "bunny"},
                    ]
                }
            }
        }),
    )?;


    std::thread::sleep(Duration::from_millis(500));

    info!("[example] Start pipeline");
    common::post("start", &json!({}))?;

    Ok(())
}
