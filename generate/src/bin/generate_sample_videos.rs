use std::{
    fs::{self},
    path::PathBuf,
    thread::{self},
    time::Duration,
};

use anyhow::Result;
use compositor_render::{event_handler::subscribe, Resolution};
use generate::compositor_instance::CompositorInstance;
use serde_json::json;

fn main() {
    let _ = fs::remove_dir_all(workingdir());
    fs::create_dir_all(workingdir()).unwrap();

    generate_video_series(
        Duration::from_secs(10),
        Resolution {
            width: 1920,
            height: 1080,
        },
        "1920x1080",
    );

    generate_video_series(
        Duration::from_secs(10),
        Resolution {
            width: 1080,
            height: 1920,
        },
        "1080x1920",
    );

    generate_video_series(
        Duration::from_secs(10),
        Resolution {
            width: 854,
            height: 480,
        },
        "854x480",
    );

    generate_video_series(
        Duration::from_secs(10),
        Resolution {
            width: 480,
            height: 854,
        },
        "480x854",
    );

    generate_video_series(
        Duration::from_secs(10),
        Resolution {
            width: 1440,
            height: 1080,
        },
        "1440x1080",
    );

    generate_video_series(
        Duration::from_secs(10),
        Resolution {
            width: 1080,
            height: 1440,
        },
        "1080x1440",
    );
}

fn generate_video_series(duration: Duration, resolution: Resolution, name_suffix: &str) {
    // HSV 240°, 50%, 65% (dark blue)
    generate_video(
        workingdir().join(format!("input_1_{}.mp4", name_suffix)),
        "Input 1",
        "#5353a6ff",
        duration,
        &resolution,
    )
    .unwrap();
    // HSV 120°, 50%, 65% (green)
    generate_video(
        workingdir().join(format!("input_2_{}.mp4", name_suffix)),
        "Input 2",
        "#53a653ff",
        duration,
        &resolution,
    )
    .unwrap();
    // HSV 0°, 50%, 65% (red)
    generate_video(
        workingdir().join(format!("input_3_{}.mp4", name_suffix)),
        "Input 3",
        "#a65353ff",
        duration,
        &resolution,
    )
    .unwrap();
    // HSV 60°, 50%, 65% (yellow)
    generate_video(
        workingdir().join(format!("input_4_{}.mp4", name_suffix)),
        "Input 4",
        "#a6a653ff",
        duration,
        &resolution,
    )
    .unwrap();
    // HSV 180°, 50%, 65% (light blue)
    generate_video(
        workingdir().join(format!("input_5_{}.mp4", name_suffix)),
        "Input 5",
        "#53a6a6ff",
        duration,
        &resolution,
    )
    .unwrap();
    // HSV 300°, 50%, 65% (purple)
    generate_video(
        workingdir().join(format!("input_6_{}.mp4", name_suffix)),
        "Input 6",
        "#a653a6ff",
        duration,
        &resolution,
    )
    .unwrap();
}

fn workingdir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("workingdir")
        .join("inputs_mp4")
}

fn generate_video(
    path: PathBuf,
    text: &str,
    rgba_color: &str,
    duration: Duration,
    resolution: &Resolution,
) -> Result<()> {
    let instance = CompositorInstance::start();

    instance.send_request(
        "output/output_1/register",
        json!({
            "type": "mp4",
            "path": format!("{}", path.to_string_lossy().to_string()),
            "video": {
                "resolution": {
                    "width": resolution.width,
                    "height": resolution.height,
                },
                "encoder": {
                    "type": "ffmpeg_h264",
                    "preset": "medium",
                    "ffmpeg_options": {
                        "crf": "32"
                    }
                },
                "initial": scene(text, rgba_color, resolution, Duration::ZERO)
            },
        }),
    )?;

    instance.send_request(
        "output/output_1/unregister",
        json!({
            "schedule_time_ms": duration.as_millis(),
        }),
    )?;

    const EVENT_COUNT: u64 = 10_000;
    for i in 0..EVENT_COUNT {
        let pts = Duration::from_millis(duration.as_millis() as u64 * i / EVENT_COUNT);
        instance.send_request(
            "output/output_1/update",
            json!({
                "video": scene(text, rgba_color, resolution, pts),
                "schedule_time_ms": pts.as_millis(),
            }),
        )?;
    }

    instance.send_request("start", json!({}))?;

    thread::spawn(|| {
        let event_receiver = subscribe();
        loop {
            if let Ok(event) = event_receiver.recv() {
                if event.kind == "OUTPUT_DONE".to_string() {
                    break;
                }
            }
        }
    })
    .join()
    .unwrap();

    Ok(())
}

fn scene(
    text: &str,
    rgba_color: &str,
    resolution: &Resolution,
    pts: Duration,
) -> serde_json::Value {
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
                { "type": "view" },
                {
                  "type": "view",
                  "bottom": resolution.height / 6,
                  "right": resolution.width / 8,
                  "width":  resolution.width / 6,
                  "height": resolution.height / 6,
                  "children": [
                     {
                            "type": "text",
                            "text": format!("{:.2}s", pts.as_millis() as f32 / 1000.0),
                            "font_size": resolution.width / 16,
                            "width": resolution.width / 6,
                            "align": "right",
                            "font_family": "Comic Sans MS",
                     },
                  ]
                }
            ]
        }
    })
}
