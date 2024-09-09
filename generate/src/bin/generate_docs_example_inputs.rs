use std::{fs, path::PathBuf, process::Command, thread, time::Duration};

use anyhow::Result;
use generate::compositor_instance::CompositorInstance;
use serde_json::json;

fn main() {
    let _ = fs::remove_dir_all(workingdir());
    fs::create_dir_all(workingdir()).unwrap();

    // HSV 240°, 50%, 65% (dark blue)
    generate_video(workingdir().join("input_1.mp4"), "Input 1", "#5353a6ff").unwrap();
    // HSV 120°, 50%, 65% (green)
    generate_video(workingdir().join("input_2.mp4"), "Input 2", "#53a653ff").unwrap();
    // HSV 0°, 50%, 65% (red)
    generate_video(workingdir().join("input_3.mp4"), "Input 3", "#a65353ff").unwrap();
    // HSV 60°, 50%, 65% (yellow)
    generate_video(workingdir().join("input_4.mp4"), "Input 4", "#a6a653ff").unwrap();
    // HSV 180°, 50%, 65% (light blue)
    generate_video(workingdir().join("input_5.mp4"), "Input 5", "#53a6a6ff").unwrap();
    // HSV 300°, 50%, 65% (purple)
    generate_video(workingdir().join("input_6.mp4"), "Input 6", "#a653a6ff").unwrap();
}

fn workingdir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("workingdir")
        .join("inputs")
}

fn generate_video(path: PathBuf, text: &str, rgba_color: &str) -> Result<()> {
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
                    "width": 1920,
                    "height": 1080,
                },
                "encoder": {
                    "type": "ffmpeg_h264",
                    "preset": "ultrafast"
                },
                "initial": scene(text, rgba_color, Duration::ZERO)
            },
        }),
    )?;

    instance.send_request(
        "output/output_1/unregister",
        json!({
            "schedule_time_ms": 20_000,
        }),
    )?;

    const EVENT_COUNT: u64 = 10_000;
    for i in 0..EVENT_COUNT {
        let pts = Duration::from_millis(20_000 * i / EVENT_COUNT);
        instance.send_request(
            "output/output_1/update",
            json!({
                "video": scene(text, rgba_color, pts),
                "schedule_time_ms": pts.as_millis(),
            }),
        )?;
    }

    let gst_thread = thread::Builder::new().name("gst sink".to_string()).spawn(move  ||{
        let gst_cmd = format!(
            "gst-launch-1.0 -v tcpclientsrc host=127.0.0.1 port={} ! \"application/x-rtp-stream\" ! x264enc ! mp4mux ! filesink location={}",
            output_port,
            path.to_string_lossy(),
        );
        Command::new("bash").arg("-c").arg(gst_cmd).status().unwrap();
    }).unwrap();

    instance.send_request("start", json!({}))?;

    gst_thread.join().unwrap();

    Ok(())
}

fn scene(text: &str, rgba_color: &str, pts: Duration) -> serde_json::Value {
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
                    "font_size": 250,
                    "width": 1920,
                    "align": "center",
                    "font_family": "Comic Sans MS",
                },
                { "type": "view" },
                {
                  "type": "view",
                  "bottom": 100,
                  "right": 100,
                  "width":  300,
                  "height": 100,
                  "children": [
                     {
                            "type": "text",
                            "text": format!("{:.2}s", pts.as_millis() as f32 / 1000.0),
                            "font_size": 90,
                            "width": 300,
                            "align": "right",
                            "font_family": "Comic Sans MS",
                     },
                  ]
                }
            ]
        }
    })
}
