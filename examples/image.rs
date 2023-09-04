use anyhow::Result;
use compositor_common::{scene::Resolution, Framerate};
use log::{error, info};
use serde_json::json;
use std::{
    path::PathBuf,
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
    height: 1054,
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

    info!("[example] Register static images");
    common::post(&json!({
        "type": "register",
        "entity_type": "image",
        "image_id": "example_gif",
        "asset_type": "gif",
        "url": "https://gifdb.com/images/high/rust-logo-on-fire-o41c0v9om8drr8dv.gif",
    }))?;
    common::post(&json!({
        "type": "register",
        "entity_type": "image",
        "image_id": "example_jpeg",
        "asset_type": "jpeg",
        "url": "https://www.rust-lang.org/static/images/rust-social.jpg",
    }))?;
    common::post(&json!({
        "type": "register",
        "entity_type": "image",
        "asset_type": "svg",
        "image_id": "example_svg",
        "path": PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/assets/rust.svg"),
    }))?;
    common::post(&json!({
        "type": "register",
        "entity_type": "image",
        "asset_type": "png",
        "image_id": "example_png",
        "url": "https://rust-lang.org/logos/rust-logo-512x512.png",
        "resolution": { "width": VIDEO_RESOLUTION.width, "height": VIDEO_RESOLUTION.height },
    }))?;

    info!("[example] Update scene");
    common::post(&json!({
        "type": "update_scene",
        "inputs": [],
        "nodes": [
            {
                 "node_id": "gif_1",
                 "type": "image",
                 "image_id": "example_gif",
            },
            {
                 "node_id": "png_1",
                 "type": "image",
                 "image_id": "example_png",
            },
            {
                 "node_id": "jpeg_1",
                 "type": "image",
                 "image_id": "example_jpeg",
            },
            {
                 "node_id": "svg_1",
                 "type": "image",
                 "image_id": "example_svg",
            },
            {
                "node_id": "gif_1_rescaled",
                "type": "built-in",
                "transformation": "transform_to_resolution",
                "strategy": "fit",
                "resolution": { "width": 960, "height": 540 },
                "input_pads": ["gif_1"],
            },
            {
                "node_id": "png_1_rescaled",
                "type": "built-in",
                "transformation": "transform_to_resolution",
                "strategy": "fit",
                "resolution": { "width": 960, "height": 540 },
                "input_pads": ["png_1"],
            },
            {
                "node_id": "jpeg_1_rescaled",
                "type": "built-in",
                "transformation": "transform_to_resolution",
                "strategy": "fit",
                "resolution": { "width": 960, "height": 540 },
                "input_pads": ["jpeg_1"],
                "background_color_rgba": "#00000000",
            },
            {
                "node_id": "svg_1_rescaled",
                "type": "built-in",
                "transformation": "transform_to_resolution",
                "strategy": "fit",
                "resolution": { "width": 960, "height": 540 },
                "input_pads": ["svg_1"],
            },
            {
                "node_id": "layout",
                "type": "built-in",
                "transformation": "fixed_position_layout",
                "texture_layouts": [
                    {
                        "left": "0px",
                        "top": "0px"
                    },
                    {
                        "left": "960px",
                        "top": "540px"
                    },
                    {
                        "left": "0px",
                        "top": "540px"
                    },
                    {
                        "left": "960px",
                        "top": "0px",
                    },
                ],
                "background_color_rgba": "#0000FF00",
                "resolution": { "width": VIDEO_RESOLUTION.width, "height": VIDEO_RESOLUTION.height },
                "input_pads": ["gif_1_rescaled", "png_1_rescaled", "jpeg_1_rescaled", "svg_1_rescaled"],
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

    Ok(())
}
