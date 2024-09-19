use anyhow::Result;
use compositor_api::types::Resolution;
use serde_json::json;
use std::time::Duration;

use integration_tests::{
    examples::{self, run_example, TestSample},
    ffmpeg::{start_ffmpeg_receive, start_ffmpeg_send},
};

const VIDEO_RESOLUTION: Resolution = Resolution {
    width: 1280,
    height: 720,
};

const IP: &str = "127.0.0.1";
const INPUT_PORT: u16 = 8002;
const OUTPUT_PORT: u16 = 8004;

const VIDEOS: u16 = 6;

fn main() {
    run_example(client_code);
}

fn client_code() -> Result<()> {
    start_ffmpeg_receive(Some(OUTPUT_PORT), None)?;

    let mut children = Vec::new();

    for i in 1..VIDEOS + 1 {
        let input_name = format!("input_{i}");

        examples::post(
            &format!("input/{input_name}/register"),
            &json!({
                    "type": "rtp_stream",
                    "port": INPUT_PORT + 2 + 2 * i,
                    "video": {
                    "decoder": "vulkan_video"
                }
            }),
        )?;

        children.push(json!({
            "type": "input_stream",
            "input_id": input_name,
        }));
    }

    let scene = json!({
        "type": "tiles",
        "id": "tile",
        "padding": 5,
        "background_color_rgba": "#444444FF",
        "children": children,
        "transition": {
            "duration_ms": 700,
            "easing_function": {
                "function_name": "cubic_bezier",
                "points": [0.35, 0.22, 0.1, 0.8]
            }
        },
    });

    let shader_source = include_str!("./silly.wgsl");
    examples::post(
        "shader/shader_example_1/register",
        &json!({
            "source": shader_source,
        }),
    )?;

    examples::post(
        "output/output_1/register",
        &json!({
            "type": "rtp_stream",
            "port": OUTPUT_PORT,
            "ip": IP,
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
                    "root": scene
                }
            }
        }),
    )?;

    std::thread::sleep(Duration::from_millis(500));

    examples::post("start", &json!({}))?;

    for i in 1..VIDEOS + 1 {
        start_ffmpeg_send(
            IP,
            Some(INPUT_PORT + 2 + 2 * i),
            None,
            TestSample::BigBuckBunny,
        )?;
    }
    Ok(())
}
