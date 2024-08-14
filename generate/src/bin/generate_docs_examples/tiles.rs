use anyhow::Result;
use generate::compositor_instance::CompositorInstance;
use serde_json::json;

use crate::{component_asset_path, run_generate_example};

pub(super) fn generate_tiles() -> Result<()> {
    let request_sender = |instance: &CompositorInstance, output_port: u16| {
        instance.send_request(
            "output/output_1/register",
            json!({
                "type": "rtp_stream",
                "transport_protocol": "tcp_server",
                "port": output_port,
                "video": {
                    "resolution": {
                        "width": 1280,
                        "height": 720,
                    },
                    "encoder": {
                        "type": "ffmpeg_h264",
                        "preset": "ultrafast"
                    },
                    "initial": scene(vec!["input_1", "input_2"])
                },
            }),
        )?;
        instance.send_request(
            "output/output_1/update",
            json!({
                "video": scene(vec!["input_1", "input_2", "input_3"]),
                "schedule_time_ms": 2000
            }),
        )?;
        instance.send_request(
            "output/output_1/update",
            json!({
                "video": scene(vec!["input_1", "input_2", "input_3", "input_4"]),
                "schedule_time_ms": 4000
            }),
        )?;
        instance.send_request(
            "output/output_1/update",
            json!({
                "video": scene(vec!["input_1", "input_2", "input_3", "input_4", "input_5"]),
                "schedule_time_ms": 6000
            }),
        )?;
        instance.send_request(
            "output/output_1/update",
            json!({
                "video": scene(vec!["input_1", "input_2", "input_3", "input_5", "input_4"]),
                "schedule_time_ms": 8000
            }),
        )?;
        instance.send_request(
            "output/output_1/update",
            json!({
                "video": scene(vec!["input_1", "input_2", "input_4"]),
                "schedule_time_ms": 10_000
            }),
        )?;
        instance.send_request(
            "output/output_1/update",
            json!({
                "video": scene(vec!["input_1", "input_2"]),
                "schedule_time_ms": 12_000
            }),
        )?;
        instance.send_request(
            "output/output_1/unregister",
            json!({
                "schedule_time_ms": 14_000,
            }),
        )?;

        Ok(())
    };

    run_generate_example(component_asset_path("tile_transitions.webp"), request_sender)?;
    Ok(())
}

fn scene(inputs: Vec<&str>) -> serde_json::Value {
    let inputs = inputs
        .into_iter()
        .map(|id| {
            json!({
                "type": "input_stream",
                "input_id": id,
                "id": id,
            })
        })
        .collect::<Vec<_>>();
    json!({
        "root": {
            "type": "tiles",
            "id": "tile",
            "children": inputs,
            "margin": 20,
            "background_color_rgba": "#4d4d4dff",
            "transition": {
                "duration_ms": 500,
                "easing_function": {
                    "function_name": "cubic_bezier",
                    "points": [0.35, 0.22, 0.1, 0.8]
                }
            },
        }
    })
}
