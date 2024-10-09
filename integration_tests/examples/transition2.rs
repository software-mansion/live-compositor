use anyhow::Result;
use compositor_api::types::Resolution;
use serde_json::json;
use std::{
    thread::{self},
    time::Duration,
};

use integration_tests::{
    examples::{self, run_example, TestSample},
    ffmpeg::{start_ffmpeg_receive, start_ffmpeg_send},
};

const VIDEO_RESOLUTION: Resolution = Resolution {
    width: 640,
    height: 360,
};

const IP: &str = "127.0.0.1";
const INPUT_PORT: u16 = 8002;
const OUTPUT_PORT: u16 = 8004;

fn main() {
    run_example(client_code);
}

fn client_code() -> Result<()> {
    start_ffmpeg_receive(Some(OUTPUT_PORT), None)?;

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

    examples::post(
        "image/example_image/register",
        &json!({
            "asset_type": "gif",
            "url": "https://gifdb.com/images/high/rust-logo-on-fire-o41c0v9om8drr8dv.gif",
        }),
    )?;

    let scene1 = json!({
        "type": "view",
        "background_color_rgba": "#42daf5ff",
        "children": [
            {
                "type": "view",
                "id": "resized",
                "width": VIDEO_RESOLUTION.width - 160,
                "height": VIDEO_RESOLUTION.height - 90,
                "top": 20.5,
                "right": 20.5,
                "border_width": 5,
                "border_radius": 30,
                "background_color_rgba": "#0000FFFF",
                "border_color_rgba": "#FFFFFFFF",
                "box_shadows": [
                    {
                        "offset_y": 60,
                        "offset_x": -60,
                        "blur_radius": 60,
                        "color_rgba": "#FFFF00FF",
                    }
                ],
                "children": [
                    {
                        "type": "rescaler",
                        "child": {
                            "type": "input_stream",
                            "input_id": "input_1"
                        }
                    }
                ]
            }
        ]
    });

    let scene2 = json!({
        "type": "view",
        "background_color_rgba": "#42daf5ff",
        "children": [
            {
                "type": "view",
                "id": "resized",
                "width": VIDEO_RESOLUTION.width - 160*2,
                "height": VIDEO_RESOLUTION.height - 90*2,
                "top": 200.5,
                "right": 200.5,
                "border_radius": 50,
                "border_width": 5,
                "border_color_rgba": "#FFFF00FF",
                "background_color_rgba": "#0000FFFF",
                "box_shadows": [
                    {
                        "offset_y": 60,
                        "offset_x": -60,
                        "blur_radius": 40,
                        "color_rgba": "#FFFF00FF",
                    }
                ],
                "transition": {
                    "duration_ms": 10000
                },
                "children": [
                    {
                        "type": "rescaler",
                        "child": {
                            "type": "input_stream",
                            "input_id": "input_1"
                        }
                    }
                ]
            }
        ]
    });

    examples::post(
        "output/output_1/register",
        &json!({
            //"type": "mp4",
            //"path": "smooth2.mp4",
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

    examples::post("start", &json!({}))?;

    start_ffmpeg_send(IP, Some(INPUT_PORT), None, TestSample::TestPattern)?;

    thread::sleep(Duration::from_secs(5));

    examples::post(
        "output/output_1/update",
        &json!({
            "video": {
                "root": scene2,
            }
        }),
    )?;

    //   sleep(Duration::from_secs(12));
    //  examples::post("output/output_1/unregister", &json!({}))?;

    Ok(())
}
