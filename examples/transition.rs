use anyhow::Result;
use log::{error, info};
use serde_json::json;
use std::{thread, time::Duration};
use video_compositor::{logger, server, types::Resolution};

use crate::common::{start_ffplay, start_websocket_thread, stream_ffmpeg_testsrc};

#[path = "./common/common.rs"]
mod common;

const VIDEO_RESOLUTION: Resolution = Resolution {
    width: 1920,
    height: 1080,
};

const IP: &str = "127.0.0.1";
const INPUT_PORT: u16 = 8002;
const OUTPUT_PORT: u16 = 8004;

fn main() {
    ffmpeg_next::format::network::init();
    logger::init_logger();

    thread::spawn(|| {
        if let Err(err) = start_example_client_code() {
            error!("{err}")
        }
    });

    server::run()
}

fn start_example_client_code() -> Result<()> {
    info!("[example] Start listening on output port.");
    start_ffplay(IP, OUTPUT_PORT, None)?;
    start_websocket_thread();

    info!("[example] Send register input request.");
    common::post(&json!({
        "type": "register",
        "entity_type": "rtp_input_stream",
        "input_id": "input_1",
        "port": INPUT_PORT,
        "video": {
            "codec": "h264"
        }
    }))?;

    let shader_source = include_str!("./silly.wgsl");
    info!("[example] Register shader transform");
    common::post(&json!({
        "type": "register",
        "entity_type": "shader",
        "shader_id": "example_shader",
        "source": shader_source,
    }))?;

    info!("[example] Register static image");
    common::post(&json!({
        "type": "register",
        "entity_type": "image",
        "image_id": "example_image",
        "asset_type": "gif",
        "url": "https://gifdb.com/images/high/rust-logo-on-fire-o41c0v9om8drr8dv.gif",
    }))?;

    let scene1 = json!({
        "type": "view",
        "background_color_rgba": "#444444FF",
        "children": [
            {
                "type": "view",
                "id": "resized",
                "overflow": "fit",
                "width": 480,
                "height": 270,
                "top": 100,
                "right": 100,
                "children": [
                    {
                        "type": "shader",
                        "shader_id": "example_shader",
                        "resolution": { "width": VIDEO_RESOLUTION.width, "height": VIDEO_RESOLUTION.height },
                        "children": [{
                            "type": "input_stream",
                            "input_id": "input_1",
                        }]
                    }
                ]
            }
        ]
    });

    let scene2 = json!({
        "type": "view",
        "background_color_rgba": "#444444FF",
        "children": [
            {
                "type": "view",
                "id": "resized",
                "width": VIDEO_RESOLUTION.width,
                "height": VIDEO_RESOLUTION.height,
                "top": 0,
                "right": 0,
                "transition": {
                    "duration_ms": 10000
                },
                "children": [
                    {
                        "type": "rescaler",
                        "mode": "fit",
                        "child": {
                            "type": "shader",
                            "shader_id": "example_shader",
                            "resolution": { "width": VIDEO_RESOLUTION.width, "height": VIDEO_RESOLUTION.height },
                            "children": [{
                                "type": "input_stream",
                                "input_id": "input_1",
                            }]
                        }

                    }
                ]
            }
        ]
    });

    let scene3 = json!({
        "type": "view",
        "background_color_rgba": "#444444FF",
        "children": [
            {
                "type": "view",
                "id": "resized",
                "width": VIDEO_RESOLUTION.width,
                "height": VIDEO_RESOLUTION.height,
                "top": 0,
                "right": 0,
                "children": [
                    {
                        "type": "rescaler",
                        "mode": "fit",
                        "child": {
                            "type": "shader",
                            "shader_id": "example_shader",
                            "resolution": { "width": VIDEO_RESOLUTION.width, "height": VIDEO_RESOLUTION.height },
                            "children": [{
                                "type": "input_stream",
                                "input_id": "input_1",
                            }]
                        }

                    }
                ]
            }
        ]
    });

    info!("[example] Send register output request.");
    common::post(&json!({
        "type": "register",
        "entity_type": "output_stream",
        "output_id": "output_1",
        "ip": IP,
        "port": OUTPUT_PORT,
        "video": {
            "resolution": {
                "width": VIDEO_RESOLUTION.width,
                "height": VIDEO_RESOLUTION.height,
            },
            "encoder_preset": "ultrafast",
            "initial": scene1
        }
    }))?;

    info!("[example] Start pipeline");
    common::post(&json!({
        "type": "start",
    }))?;

    info!("[example] Start input stream");
    stream_ffmpeg_testsrc(IP, OUTPUT_PORT, VIDEO_RESOLUTION)?;

    thread::sleep(Duration::from_secs(5));

    info!("[example] Update output");
    common::post(&json!({
        "type": "update_output",
        "output_id": "output_1",
        "video": scene2,
    }))?;

    thread::sleep(Duration::from_secs(2));

    info!("[example] Update output");
    common::post(&json!({
        "type": "update_output",
        "output_id": "output_1",
        "video": scene3,
    }))?;

    Ok(())
}
