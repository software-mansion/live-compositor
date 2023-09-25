use anyhow::Result;
use compositor_common::{scene::Resolution, Framerate};
use log::{error, info};
use serde_json::json;
use std::{
    process::{Command, Stdio},
    thread,
    time::Duration,
};
use video_compositor::http;

use crate::common::write_example_sdp_file;

#[path = "./common/common.rs"]
mod common;

const VIDEO_RESOLUTION: Resolution = Resolution {
    width: 1920,
    height: 1080,
};
const FRAMERATE: Framerate = Framerate(30);

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
        "web_renderer": {
            "init": false
        },
        "framerate": FRAMERATE,
        "stream_fallback_timeout_ms": 2000
    }))?;

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

    let shader_source = include_str!("../compositor_render/examples/silly/silly.wgsl");
    info!("[example] Register shader transform");
    common::post(&json!({
        "type": "register",
        "entity_type": "shader",
        "shader_id": "example_shader",
        "source": shader_source,
        "fallback_strategy": "fallback_if_all_inputs_missing",
        "constraints": [
            {
                "type": "input_count",
                "fixed_count": 1,
            }
        ]
    }))?;

    info!("[example] Register static image");
    common::post(&json!({
        "type": "register",
        "entity_type": "image",
        "image_id": "example_image",
        "asset_type": "gif",
        "url": "https://gifdb.com/images/high/rust-logo-on-fire-o41c0v9om8drr8dv.gif",
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

    info!("[example] Update scene");
    common::post(&json!({
        "type": "update_scene",
        "nodes": [
            {
                "type": "image",
                "node_id": "image",
                "image_id": "example_image",
            },
            {
                "type": "shader",
                "node_id": "shader_1",
                "shader_id": "example_shader",
                "fallback_id": "image",
                "input_pads": [
                    "input_1",
                ],
                "resolution": { "width": 960, "height": 540 },
            },
            {
                "type": "transition",
                "node_id": "transition_1",
                "start": {
                    "transformation": "fixed_position_layout",
                    "texture_layouts": [
                        {
                            "left": "0px",
                            "top": "0px"
                        },
                    ],
                    "background_color_rgba": "#0000FF00",
                    "resolution": { "width": VIDEO_RESOLUTION.width, "height": VIDEO_RESOLUTION.height },
                },
                "end": {
                    "transformation": "fixed_position_layout",
                    "texture_layouts": [
                        {
                            "right": "0px",
                            "bottom": "0px",
                            "rotation": 180,
                        },
                    ],
                    "background_color_rgba": "#0000FF00",
                    "resolution": { "width": VIDEO_RESOLUTION.width, "height": VIDEO_RESOLUTION.height },
                },
                "interpolation": "spring",
                "transition_duration_ms": 10_000,
                "input_pads": ["shader_1"],
            }
        ],
        "outputs": [
            {
                "output_id": "output_1",
                "input_pad": "transition_1"
            }
        ]
    }))?;

    Ok(())
}
