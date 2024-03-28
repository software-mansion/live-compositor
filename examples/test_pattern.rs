use anyhow::Result;
use log::{error, info};
use serde::Deserialize;
use serde_json::json;
use std::{env, thread};
use video_compositor::{server, types::Resolution};

use crate::common::{start_ffplay, start_websocket_thread, stream_ffmpeg_testsrc};

#[path = "./common/common.rs"]
mod common;

const VIDEO_RESOLUTION: Resolution = Resolution {
    width: 1920,
    height: 1080,
};

const IP: &str = "127.0.0.1";
const OUTPUT_PORT: u16 = 8002;

fn main() {
    env::set_var("LIVE_COMPOSITOR_WEB_RENDERER_ENABLE", "0");
    ffmpeg_next::format::network::init();

    thread::spawn(|| {
        if let Err(err) = start_example_client_code() {
            error!("{err}")
        }
    });

    server::run()
}

#[derive(Deserialize)]
struct RegisterResponse {
    port: u16,
}

fn start_example_client_code() -> Result<()> {
    info!("[example] Start listening on output port.");
    start_ffplay(IP, OUTPUT_PORT, None)?;
    start_websocket_thread();

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
        "port": OUTPUT_PORT,
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
    stream_ffmpeg_testsrc(IP, input_port, VIDEO_RESOLUTION)?;

    info!("[example] Start pipeline");
    common::post(&json!({
        "type": "start",
    }))?;

    Ok(())
}
