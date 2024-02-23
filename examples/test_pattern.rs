use anyhow::Result;
use log::{error, info};
use serde::Deserialize;
use serde_json::json;
use std::{
    env,
    process::{Command, Stdio},
    thread,
    time::Duration,
};
use video_compositor::{config::config, http, logger, types::Resolution};

use crate::common::write_video_example_sdp_file;

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

#[derive(Deserialize)]
struct RegisterResponse {
    port: u16,
}

fn start_example_client_code() -> Result<()> {
    info!("[example] Start listening on output port.");
    let output_sdp = write_video_example_sdp_file("127.0.0.1", 8002)?;
    Command::new("ffplay")
        .args(["-protocol_whitelist", "file,rtp,udp", &output_sdp])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    thread::sleep(Duration::from_secs(2));

    info!("[example] Send register input request.");
    let RegisterResponse { port: input_port } = common::post(&json!({
        "type": "register",
        "entity_type": "rtp_input_stream",
        "input_id": "input_1",
        "port": "8004:8008",
        "video": {
            "codec": "h264"
        },
        "offset_ms": 0,
        "required": true,
    }))?
    .json::<RegisterResponse>()?;

    let shader_source = include_str!("./silly.wgsl");
    info!("[example] Register shader transform");
    common::post(&json!({
        "type": "register",
        "entity_type": "shader",
        "shader_id": "example_shader",
        "source": shader_source,
    }))?;

    let scene = json!( {
        "type": "shader",
        "id": "shader_1",
        "shader_id": "example_shader",
        "children": [
            {
                "type": "input_stream",
                "input_id": "input_1",
            }
        ],
        "resolution": { "width": VIDEO_RESOLUTION.width, "height": VIDEO_RESOLUTION.height },
    });

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
            "initial": scene
        }
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
            &format!("rtp://127.0.0.1:{}?rtcpport={}", input_port, input_port),
        ])
        .spawn()?;

    info!("[example] Start pipeline");
    common::post(&json!({
        "type": "start",
    }))?;

    Ok(())
}
