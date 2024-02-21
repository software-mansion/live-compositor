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

use crate::common::write_example_sdp_file;

#[path = "./common/common.rs"]
mod common;

const VIDEO_RESOLUTION: Resolution = Resolution {
    width: 1920,
    height: 1080,
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
    let output_sdp = write_example_sdp_file("127.0.0.1", 8002)?;
    Command::new("ffplay")
        .args(["-protocol_whitelist", "file,rtp,udp", &output_sdp])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    thread::sleep(Duration::from_secs(2));

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
            "initial": scene_with_inputs(0)
        }
    }))?;

    for i in 1..=16 {
        info!("[example] Update scene");
        common::post(&json!({
            "type": "update_scene",
            "output_id": "output_1",
            "scene": scene_with_inputs(i),
            "schedule_time_ms": i * 1000,
        }))?;
    }

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

    for i in 0..16 {
        info!("[example] Update scene");
        common::post(&json!({
            "type": "update_output",
            "output_id": "output_1",
            "scene": scene_with_inputs(16 - i),
            "schedule_time_ms": (20 + i) * 1000,
        }))?;
    }

    info!("[example] Update output");
    common::post(&json!({
        "type": "update_output",
        "output_id": "output_1",
        "scene": scene_with_inputs(4),
        "schedule_time_ms": 40 * 1000,
    }))?;

    Ok(())
}
