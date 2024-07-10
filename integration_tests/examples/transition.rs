use anyhow::Result;
use live_compositor::{server, types::Resolution};
use log::{error, info};
use serde_json::json;
use std::{thread, time::Duration};

use integration_tests::examples::{
    self, start_ffplay, start_websocket_thread, stream_ffmpeg_testsrc,
};

const VIDEO_RESOLUTION: Resolution = Resolution {
    width: 1920,
    height: 1080,
};

const IP: &str = "127.0.0.1";
const INPUT_PORT: u16 = 8002;
const OUTPUT_PORT: u16 = 8004;

fn main() {
    ffmpeg_next::format::network::init();

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
    examples::post(
        "input/input_1/register",
        &json!({
            "type": "rtp_stream",
            "port": INPUT_PORT,
            "video": {
                "decoder": "ffmpeg_h264"
            }
        }),
    )?;

    let shader_source = include_str!("./silly.wgsl");
    info!("[example] Register shader transform");
    examples::post(
        "shader/example_shader/register",
        &json!({
            "source": shader_source,
        }),
    )?;

    info!("[example] Register static image");
    examples::post(
        "image/example_image/register",
        &json!({
            "asset_type": "gif",
            "url": "https://gifdb.com/images/high/rust-logo-on-fire-o41c0v9om8drr8dv.gif",
        }),
    )?;

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
    examples::post(
        "output/output_1/register",
        &json!({
            "type": "rtp_stream",
            "ip": IP,
            "port": OUTPUT_PORT,
            "video": {
                "resolution": {
                    "width": VIDEO_RESOLUTION.width,
                    "height": VIDEO_RESOLUTION.height,
                },
                "encoder": {
                    "type": "ffmpeg_h264",
                    "preset": "ultrafast"
                },
                "initial": {
                    "root": scene1
                }
            }
        }),
    )?;

    info!("[example] Start pipeline");
    examples::post("start", &json!({}))?;

    info!("[example] Start input stream");
    stream_ffmpeg_testsrc(IP, INPUT_PORT, VIDEO_RESOLUTION)?;

    thread::sleep(Duration::from_secs(5));

    info!("[example] Update output");
    examples::post(
        "output/output_1/update",
        &json!({
            "video": {
                "root": scene2,
            }
        }),
    )?;

    thread::sleep(Duration::from_secs(2));

    info!("[example] Update output");
    examples::post(
        "output/output_1/update",
        &json!({
            "video": {
                "root": scene3
            },
        }),
    )?;

    Ok(())
}
