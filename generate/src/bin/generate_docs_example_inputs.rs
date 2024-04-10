use std::{fs, path::PathBuf, process::Command, thread, time::Duration};

use anyhow::Result;
use generate::compositor_instance::CompositorInstance;
use serde_json::json;

fn main() {
    let _ = fs::remove_dir_all(workingdir());
    fs::create_dir_all(workingdir()).unwrap();
    generate_video(workingdir().join("input_1.rtp"), "Input 1", "#4d4d4dff").unwrap();
    generate_video(workingdir().join("input_2.rtp"), "Input 2", "#9999ffff").unwrap();
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
            "schedule_time_ms": 10_000,
        }),
    )?;

    const EVENT_COUNT: u64 = 10_000;
    for i in 0..EVENT_COUNT {
        let pts = Duration::from_millis(10_000 * i / EVENT_COUNT);
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
            "gst-launch-1.0 -v tcpclientsrc host=127.0.0.1 port={} ! \"application/x-rtp-stream\" ! filesink location={}",
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
                    "font_size": 200,
                    "width": 1920,
                    "align": "center",
                    "font_family": "Comic Sans MS",
                },
                { "type": "view" },
                {
                  "type": "view",
                  "bottom": 50,
                  "right": 50,
                  "width":  200,
                  "height": 50,
                  "children": [
                     {
                            "type": "text",
                            "text": format!("{:.2}s", pts.as_millis() as f32 / 1000.0),
                            "font_size": 50,
                            "font_family": "Comic Sans MS",
                     },
                  ]
                }
            ]
        }
    })
}
