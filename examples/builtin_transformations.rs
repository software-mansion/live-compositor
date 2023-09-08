use anyhow::Result;
use compositor_common::{scene::Resolution, Framerate};
use log::{error, info};
use serde_json::json;
use signal_hook::{consts, iterator::Signals};
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

    let mut signals = Signals::new([consts::SIGINT]).unwrap();
    signals.forever().next();
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

    info!("[example] Register static image");
    common::post(&json!({
        "type": "register",
        "entity_type": "image",
        "image_id": "example_image",
        "asset_type": "jpeg",
        "url": "https://i.postimg.cc/CxcvtJC5/pexels-rohi-bernard-codillo-17908342.jpg",
    }))?;

    info!("[example] Update scene");
    common::post(&json!({
        "type": "update_scene",
        "nodes": [
            {
                "node_id": "image",
                "type": "image",
                "image_id": "example_image",
            },
            {
                "type": "built-in",
                "transformation": "mirror_image",
                "node_id": "mirrored_image",
                "input_pads": ["filled_image"],
            },
            {
                "node_id": "filled_image",
                "type": "built-in",
                "transformation": "transform_to_resolution",
                "strategy": "fill",
                "resolution": { "width": 960, "height": 540 },
                "input_pads": ["image"],
            },
            {
                "node_id": "corners_rounded_image",
                "type": "built-in",
                "transformation": "corners_rounding",
                "border_radius": 100,
                "input_pads": ["filled_image"]
            },
            {
                "node_id": "layout",
                "type": "built-in",
                "transformation": "tiled_layout",
                "background_color_rgba": "#FFFFFFFF",
                "resolution": { "width": VIDEO_RESOLUTION.width, "height": VIDEO_RESOLUTION.height },
                "input_pads": ["mirrored_image", "filled_image", "corners_rounded_image", "filled_image", "filled_image", "filled_image", "filled_image", "filled_image"],
            }
        ],
        "outputs": [
            {
                "output_id": "output_1",
                "input_pad": "layout"
            },
        ]
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
    Ok(())
}
