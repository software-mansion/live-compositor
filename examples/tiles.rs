use anyhow::Result;
use log::{error, info};
use serde_json::json;
use std::{
    process::{Command, Stdio},
    thread,
    time::Duration,
};
use video_compositor::{http, logger, types::Resolution};

use crate::common::write_example_sdp_file;

#[path = "./common/common.rs"]
mod common;

const VIDEO_RESOLUTION: Resolution = Resolution {
    width: 1920,
    height: 1080,
};

const FRAMERATE: u32 = 30;

fn main() {
    ffmpeg_next::format::network::init();
    logger::init_logger();

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
        "web_renderer": {
            "init": false
        },
        "framerate": FRAMERATE,
        "stream_fallback_timeout_ms": 2000
    }))?;

    thread::sleep(Duration::from_secs(1));

    info!("[example] Start listening on output port.");
    let output_sdp = write_example_sdp_file("127.0.0.1", 8002)?;
    Command::new("ffplay")
        .args(["-protocol_whitelist", "file,rtp,udp", &output_sdp])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    info!("[example] Send register output request.");
    common::post(&json!({
        "type": "register",
        "entity_type": "output_stream",
        "output_id": "output_1",
        "port": 8002,
        "ip": "127.0.0.1",
        "resolution": {
            "width": VIDEO_RESOLUTION.width,
            "height": VIDEO_RESOLUTION.height,
        },
        "encoder_settings": {
            "preset": "ultrafast"
        }
    }))?;

    info!("[example] Send register input request.");
    common::post(&json!({
        "type": "register",
        "entity_type": "input_stream",
        "input_id": "input_1",
        "port": 8004
    }))?;

    info!("[example] Start pipeline");
    common::post(&json!({
        "type": "start",
    }))?;

    info!("[example] Start input stream");
    let ffmpeg_source = format!(
        "testsrc=s={}x{}:r=30,format=yuv420p",
        VIDEO_RESOLUTION.width, VIDEO_RESOLUTION.height
    );
    Command::new("ffmpeg")
        .args([
            "-re",
            "-f",
            "lavfi",
            "-i",
            &ffmpeg_source,
            "-c:v",
            "libx264",
            "-f",
            "rtp",
            "rtp://127.0.0.1:8004?rtcpport=8004",
        ])
        .spawn()?;

    let scene_with_inputs = |n: usize| {
        let children: Vec<_> = (0..n)
            .map(|_| {
                json!({
                    "type": "input_stream",
                    "input_id": "input_1",
                })
            })
            .collect();
        json!({
            "type": "tiles",
            "id": "tile",
            "padding": 5,
            "background_color_rgba": "#444444FF",
            "children": children,
            "transition": {
                "duration_ms": 500,
            },
        })
    };

    thread::sleep(Duration::from_secs(1));

    for i in 1..=16 {
        info!("[example] Update scene");
        common::post(&json!({
            "type": "update_scene",
            "outputs": [
                {
                    "output_id": "output_1",
                    "root": scene_with_inputs(i),
                }
            ]
        }))?;

        thread::sleep(Duration::from_secs(1));
    }

    for i in (1..=16).rev() {
        info!("[example] Update scene");
        common::post(&json!({
            "type": "update_scene",
            "outputs": [
                {
                    "output_id": "output_1",
                    "root": scene_with_inputs(i),
                }
            ]
        }))?;

        thread::sleep(Duration::from_secs(1));
    }

    info!("[example] Update scene");
    common::post(&json!({
        "type": "update_scene",
        "outputs": [
            {
                "output_id": "output_1",
                "root": scene_with_inputs(4),
            }
        ]
    }))?;

    Ok(())
}
