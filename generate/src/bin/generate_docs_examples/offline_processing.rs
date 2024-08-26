use std::{fs, process::Command, thread};

use anyhow::Result;
use generate::{compositor_instance::CompositorInstance, packet_sender::PacketSender};
use serde_json::json;

use crate::{pages_dir, workingdir};

pub(super) fn generate_offline_processing_guide() -> Result<()> {
    let instance = CompositorInstance::start();
    let output_port = instance.get_port();
    let input_1_port = instance.get_port();
    let input_2_port = instance.get_port();

    instance.send_request(
        "input/input_1/register",
        json!({
            "type": "rtp_stream",
            "transport_protocol": "tcp_server",
            "port": input_1_port,
            "video": {
                "decoder": "ffmpeg_h264"
            },
            "required": true,
            "offset_ms": 0
        }),
    )?;

    instance.send_request(
        "input/input_2/register",
        json!({
            "type": "rtp_stream",
            "transport_protocol": "tcp_server",
            "port": input_2_port,
            "video": {
                "decoder": "ffmpeg_h264"
            },
            "required": true,
            "offset_ms": 5000
        }),
    )?;

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
                "initial": {
                    "root": {
                        "type": "rescaler",
                        "child": [{
                            "type": "input_stream",
                            "input_id": "input_1"
                        }]
                    }
                }
            },
        }),
    )?;

    instance.send_request(
        "output/output_1/update",
        json!({
            "video": {
                "type": "tiles",
                "background_color_rgba": "#4d4d4dff",
                "children": [
                    { "type": "input_stream", "input_id": "input_1" },
                    { "type": "input_stream", "input_id": "input_2" }
                ]
            },
            "schedule_time_ms": 5000
        }),
    )?;

    instance.send_request(
        "output/output_1/unregister",
        json!({
            "schedule_time_ms": 10_000,
        }),
    )?;

    PacketSender::new(input_1_port)
        .unwrap()
        .send(&fs::read(workingdir().join("input_1.rtp")).unwrap())
        .unwrap();
    PacketSender::new(input_2_port)
        .unwrap()
        .send(&fs::read(workingdir().join("input_2.rtp")).unwrap())
        .unwrap();

    let path = pages_dir()
        .join("guides")
        .join("assets")
        .join("offline_processing.webp");
    let gst_thread = thread::Builder::new().name("gst sink".to_string()).spawn(move  ||{
        let gst_cmd = format!(
            "gst-launch-1.0 -v tcpclientsrc host=127.0.0.1 port={} ! \"application/x-rtp-stream\" ! rtpstreamdepay ! rtph264depay ! video/x-h264,framerate=30/1 ! h264parse ! h264timestamper ! decodebin ! webpenc animated=true speed=6 quality=50 ! filesink location={}",
            output_port,
            path.to_string_lossy(),
        );
        Command::new("bash").arg("-c").arg(gst_cmd).status().unwrap();
    }).unwrap();

    instance.send_request("start", json!({}))?;

    gst_thread.join().unwrap();

    Ok(())
}
