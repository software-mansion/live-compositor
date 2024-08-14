use anyhow::Result;
use quick_start::generate_quick_start_guide;
use std::path::PathBuf;
use std::{fs, process::Command, thread};

use generate::{compositor_instance::CompositorInstance, packet_sender::PacketSender};
use layouts_guide::generate_layouts_guide;
use serde_json::json;
use tiles::generate_tiles;
use transitions::generate_transitions_guide;

mod layouts_guide;
mod quick_start;
mod tiles;
mod transitions;

fn main() {
    generate_quick_start_guide().unwrap();
    generate_layouts_guide().unwrap();
    generate_tiles().unwrap();
    generate_transitions_guide().unwrap();
}

fn workingdir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("workingdir")
        .join("inputs")
}

fn pages_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("docs")
        .join("pages")
}

fn guide_asset_path(filename: &str) -> PathBuf {
    pages_dir().join("guides").join("assets").join(filename)
}

fn component_asset_path(filename: &str) -> PathBuf {
    pages_dir()
        .join("api")
        .join("components")
        .join("assets")
        .join(filename)
}

fn generate_scene_example(path: PathBuf, scene: serde_json::Value) -> Result<()> {
    let output_config_sender = |instance: &CompositorInstance, output_port: u16| -> Result<()> {
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
                        "root": scene,
                    },
                },
            }),
        )?;
        instance.send_request(
            "output/output_1/unregister",
            json!({
                "schedule_time_ms": 10_000,
            }),
        )?;
        Ok(())
    };

    run_generate_example(path, output_config_sender)?;

    Ok(())
}

fn generate_transition_example(
    path: PathBuf,
    scene_start: serde_json::Value,
    scene_change: serde_json::Value,
) -> Result<()> {
    let output_config_sender = |instance: &CompositorInstance, output_port: u16| -> Result<()> {
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
                        "root": scene_start,
                    },
                },
            }),
        )?;

        instance.send_request(
            "output/output_1/update",
            json!({
                "video": {
                    "root": scene_change
                },
                "schedule_time_ms": 2000
            }),
        )?;

        instance.send_request(
            "output/output_1/unregister",
            json!({
                "schedule_time_ms": 8_000,
            }),
        )?;

        Ok(())
    };

    run_generate_example(path, output_config_sender)?;
    Ok(())
}

fn run_generate_example<F>(path: PathBuf, output_config_sender: F) -> Result<()>
where
    F: Fn(&CompositorInstance, u16) -> Result<()>,
{
    let instance = CompositorInstance::start();
    let output_port = instance.get_port();
    let input_1_port = instance.get_port();
    let input_2_port = instance.get_port();
    let input_3_port = instance.get_port();
    let input_4_port = instance.get_port();
    let input_5_port = instance.get_port();

    instance.send_request(
        "input/input_1/register",
        json!({
            "type": "rtp_stream",
            "transport_protocol": "tcp_server",
            "port": input_1_port,
            "video": {
                "decoder": "ffmpeg_h264"
            },
            "required": true
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
            "required": true
        }),
    )?;
    instance.send_request(
        "input/input_3/register",
        json!({
            "type": "rtp_stream",
            "transport_protocol": "tcp_server",
            "port": input_3_port,
            "video": {
                "decoder": "ffmpeg_h264"
            },
            "required": true
        }),
    )?;
    instance.send_request(
        "input/input_4/register",
        json!({
            "type": "rtp_stream",
            "transport_protocol": "tcp_server",
            "port": input_4_port,
            "video": {
                "decoder": "ffmpeg_h264"
            },
            "required": true
        }),
    )?;
    instance.send_request(
        "input/input_5/register",
        json!({
            "type": "rtp_stream",
            "transport_protocol": "tcp_server",
            "port": input_5_port,
            "video": {
                "decoder": "ffmpeg_h264"
            },
            "required": true
        }),
    )?;

    output_config_sender(&instance, output_port)?;

    PacketSender::new(input_1_port)
        .unwrap()
        .send(&fs::read(workingdir().join("input_1.rtp")).unwrap())
        .unwrap();
    PacketSender::new(input_2_port)
        .unwrap()
        .send(&fs::read(workingdir().join("input_2.rtp")).unwrap())
        .unwrap();
    PacketSender::new(input_3_port)
        .unwrap()
        .send(&fs::read(workingdir().join("input_3.rtp")).unwrap())
        .unwrap();
    PacketSender::new(input_4_port)
        .unwrap()
        .send(&fs::read(workingdir().join("input_4.rtp")).unwrap())
        .unwrap();

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
