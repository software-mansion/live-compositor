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
    width: 1280,
    height: 720,
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

    http::Server::new(8001).start();

    let mut signals = Signals::new([consts::SIGINT]).unwrap();
    signals.forever().next();
}

fn start_example_client_code() -> Result<()> {
    thread::sleep(Duration::from_secs(2));

    info!("[example] Sending init request.");
    common::post(&json!({
        "type": "init",
        "framerate": FRAMERATE,
        "init_web_renderer": false,
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
        "type": "register_output",
        "id": "output 1",
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

    info!("[example] Send register output request.");
    common::post(&json!({
        "type": "register_output",
        "id": "output 2",
        "port": 8006,
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
        "type": "register_input",
        "id": "input 1",
        "port": 8004
    }))?;

    info!("[example] Register static image");
    common::post(&json!({
        "type": "register_transformation",
        "key": "example_image",
        //"transform": {
        //    "type": "image",
        //    "asset_type": "jpeg",
        //    "url": "https://i.ytimg.com/vi/ekthcIHDt3I/maxresdefault.jpg",
        //},
        //
        // TODO: fix commented out example(after we rescale transforms)
        // This example links to 1920x1080 image so it won't work without
        // changing the output resolution
        //"transform": {
        //    "type": "image",
        //    "asset_type": "gif",
        //    "url": "https://upload.wikimedia.org/wikipedia/commons/b/b6/PM5644-1920x1080.gif",
        //},
        "transform": {
            "type": "image",
            "asset_type": "svg",
            "url": "https://upload.wikimedia.org/wikipedia/commons/c/c1/PM5644.svg",
            "resolution": { "width": VIDEO_RESOLUTION.width, "height": VIDEO_RESOLUTION.height },
        }
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
        "transforms": [
           {
                "node_id": "static_image",
                "type": "image",
                "image_id": "example_image",
           }
        ],
        "outputs": [
            {
                "output_id": "output 1",
                "input_pad": "static_image"
            },
            {
                "output_id": "output 2",
                "input_pad": "input 1"
            }
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
