use std::{fs, path::PathBuf, process::Command, thread};

use anyhow::Result;
use compositor_render::Resolution;
use generate::compositor_instance::CompositorInstance;
use serde_json::json;

fn main() {
    let _ = fs::remove_dir_all(workingdir());
    fs::create_dir_all(workingdir()).unwrap();

    let horizontal_16x9_1080 = Resolution {
        width: 1920,
        height: 1080,
    };
    generate_input_series(horizontal_16x9_1080, "horizontal_16x9_1080");

    let vertical_9x16_1080 = Resolution {
        width: 1080,
        height: 1920,
    };
    generate_input_series(vertical_9x16_1080, "vertical_9x16_1080");

    let horizontal_16x9_480 = Resolution {
        width: 854,
        height: 480,
    };
    generate_input_series(horizontal_16x9_480, "horizontal_16x9_480");

    let vertical_9x16_480 = Resolution {
        width: 480,
        height: 854,
    };
    generate_input_series(vertical_9x16_480, "vertical_9x16_480");

    let horizontal_4x3_1080 = Resolution {
        width: 1440,
        height: 1080,
    };
    generate_input_series(horizontal_4x3_1080, "horizontal_4x3_1080");

    let vertical_3x4_1080 = Resolution {
        width: 1080,
        height: 1440,
    };
    generate_input_series(vertical_3x4_1080, "vertical_3x4_1080");
}

fn generate_input_series(resolution: Resolution, name_suffix: &str) {
    // HSV 240°, 50%, 65% (dark blue)
    generate_png(
        workingdir().join(format!("input_1_{}.png", name_suffix)),
        "Input 1",
        "#5353a6ff",
        resolution,
    )
    .unwrap();
    // HSV 120°, 50%, 65% (green)
    generate_png(
        workingdir().join(format!("input_2_{}.png", name_suffix)),
        "Input 2",
        "#53a653ff",
        resolution,
    )
    .unwrap();
    // HSV 0°, 50%, 65% (red)
    generate_png(
        workingdir().join(format!("input_3_{}.png", name_suffix)),
        "Input 3",
        "#a65353ff",
        resolution,
    )
    .unwrap();
    // HSV 60°, 50%, 65% (yellow)
    generate_png(
        workingdir().join(format!("input_4_{}.png", name_suffix)),
        "Input 4",
        "#a6a653ff",
        resolution,
    )
    .unwrap();
    // HSV 180°, 50%, 65% (light blue)
    generate_png(
        workingdir().join(format!("input_5_{}.png", name_suffix)),
        "Input 5",
        "#53a6a6ff",
        resolution,
    )
    .unwrap();
    // HSV 300°, 50%, 65% (purple)
    generate_png(
        workingdir().join(format!("input_6_{}.png", name_suffix)),
        "Input 6",
        "#a653a6ff",
        resolution,
    )
    .unwrap();
}

fn workingdir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("workingdir")
        .join("playground_inputs")
}

fn generate_png(path: PathBuf, text: &str, rgba_color: &str, resolution: Resolution) -> Result<()> {
    let instance = CompositorInstance::start();
    let output_port = instance.get_port();

    instance.send_request(
        "output/output_1/register",
        json!({
            "type": "rtp_stream",
            "transport_protocol": "tcp_server",
            "port": output_port,
            "video": {
                "resolution": {
                    "width": resolution.width,
                    "height": resolution.height,
                },
                "encoder": {
                    "type": "ffmpeg_h264",
                    "preset": "ultrafast"
                },
                "initial": scene(text, rgba_color, resolution)
            },
        }),
    )?;

    let gst_thread = thread::Builder::new().name("gst sink".to_string()).spawn(move  ||{
        let gst_cmd = format!(
            "gst-launch-1.0 -v tcpclientsrc host=127.0.0.1 port={} ! \"application/x-rtp-stream\" ! rtpstreamdepay ! rtph264depay ! video/x-h264,framerate=30/1 ! h264parse ! h264timestamper ! decodebin ! videoconvert ! pngenc snapshot=true ! filesink location={}",
            output_port,
            path.to_string_lossy(),
        );
        Command::new("bash").arg("-c").arg(gst_cmd).status().unwrap();
    }).unwrap();

    instance.send_request("start", json!({}))?;

    gst_thread.join().unwrap();

    Ok(())
}

fn scene(text: &str, rgba_color: &str, resolution: Resolution) -> serde_json::Value {
    json!({
        "root": {
            "type": "view",
            "background_color_rgba": rgba_color,
            "direction": "column",
            "children": [
                { "type": "view" },
                {
                    "type": "text",
                    "text": text,
                    "font_size": resolution.width / 8,
                    "width": resolution.width,
                    "align": "center",
                    "font_family": "Comic Sans MS",
                },
                { "type": "view" }
            ]
        }
    })
}
