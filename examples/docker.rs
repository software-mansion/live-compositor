use anyhow::{anyhow, Result};
use compositor_common::{scene::Resolution, Framerate};
use log::{error, info, warn};
use serde_json::json;
use signal_hook::{consts, iterator::Signals};
use std::{
    env,
    process::{Command, Stdio},
    thread,
    time::Duration,
};

use crate::common::write_example_sdp_file;

#[path = "./common/common.rs"]
mod common;

const VIDEO_RESOLUTION: Resolution = Resolution {
    width: 1920,
    height: 1080,
};
const FRAMERATE: Framerate = Framerate(30);

fn main() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let Ok(host_ip) = env::var("DOCKER_HOST_IP") else {
        error!("DOCKER_HOST_IP is not specified. You can find ip using 'ip addr show docker0'.");
        return;
    };

    let skip_build = env::var("SKIP_DOCKER_REBUILD").is_ok();

    build_and_start_docker(skip_build).unwrap();

    thread::spawn(|| {
        if let Err(err) = start_example_client_code(host_ip) {
            error!("{err}")
        }
    });

    let mut signals = Signals::new([consts::SIGINT]).unwrap();
    signals.forever().next();
}

fn build_and_start_docker(skip_build: bool) -> Result<()> {
    if !skip_build {
        info!("[example] docker build");
        let mut process = Command::new("docker")
            .args(["build", "-t", "video-compositor", "."])
            .spawn()?;
        let exit_code = process.wait()?;
        if Some(0) != exit_code.code() {
            return Err(anyhow!("Docker build finished with exit code {exit_code}"));
        }
    } else {
        warn!("Skipping image build, using old version.")
    }

    info!("[example] docker run");
    Command::new("docker")
        .args([
            "run",
            "-it",
            "--device",
            "/dev/dri", // expose gpu to container
            "--cap-add",
            "SYS_ADMIN", // or enable suid based namespacing
            "-p",
            "8004:8004/udp",
            "-p",
            "8001:8001",
            "video-compositor",
        ])
        .spawn()?;
    Ok(())
}

fn start_example_client_code(host_ip: String) -> Result<()> {
    thread::sleep(Duration::from_secs(5));

    info!("[example] Sending init request.");
    common::post(&json!({
        "type": "init",
        "framerate": FRAMERATE,
        "web_renderer": {
            "init": false
        },
    }))?;

    info!("[example] Start listening on output port.");
    let output_sdp = write_example_sdp_file(&host_ip, 8002)?;
    Command::new("ffplay")
        .args(["-protocol_whitelist", "file,rtp,udp", &output_sdp])
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .spawn()?;

    info!("[example] Send register output request.");
    common::post(&json!({
        "type": "register_output",
        "id": "output 1",
        "port": 8002,
        "ip": host_ip,
        "resolution": {
            "width": VIDEO_RESOLUTION.width,
            "height": VIDEO_RESOLUTION.height,
        },
        "encoder_settings": {
            "preset": "ultrafast"
        }
    }))?;

    info!("[example] Send register input request.");
    common::post(&json!({
        "type": "register_input",
        "id": "input 1",
        "port": 8004,
    }))?;

    let shader_source = include_str!("../compositor_render/examples/silly.wgsl");
    info!("[example] Register shader transform");
    common::post(&json!({
        "type": "register_transformation",
        "key": "example shader",
        "transform": {
            "type": "shader",
            "source": shader_source,
        }
    }))?;

    //info!("[example] Register web renderer transform");
    //common::post(&json!({
    //    "type": "register_transformation",
    //    "key": "example website",
    //    "transform": {
    //        "type": "web_renderer",
    //        "url": "https://www.twitch.tv/", // or other way of providing source
    //        "resolution": { "width": VIDEO_RESOLUTION.width, "height": VIDEO_RESOLUTION.height },
    //    }
    //}))?;

    info!("[example] Update scene");
    common::post(&json!({
        "type": "update_scene",
        "inputs": [
            {
                "input_id": "input 1",
                "resolution": { "width": VIDEO_RESOLUTION.width, "height": VIDEO_RESOLUTION.height },
            }
        ],
        "transforms": [
           {
                "node_id": "side-by-side",
                "type": "shader",
                "shader_id": "example shader",
                "input_pads": [
                    "input 1",
                ],
                "resolution": { "width": VIDEO_RESOLUTION.width, "height": VIDEO_RESOLUTION.height },
           },
           //  {
           //      "node_id": "add-overlay",
           //      "type": "web_renderer",
           //      "renderer_id": "example website",
           //      "input_pads": [
           //          "input 1",
           //      ],
           //      "resolution": { "width": VIDEO_RESOLUTION.width, "height": VIDEO_RESOLUTION.height },
           //  }
        ],
        "outputs": [
            {
                "output_id": "output 1",
                "input_pad": "side-by-side"
            }
        ]
    }))?;

    info!("[example] Start pipeline");
    common::post(&json!({
        "type": "start",
    }))?;

    info!("[example] Start input stream");
    let ffmpeg_source = format!(
        "testsrc=s={}x{}:r=30,format=yuv420p",
        VIDEO_RESOLUTION.width, VIDEO_RESOLUTION.height
    );

    Command::new("ffmpeg")
        .args([
            "-re",
            "-f",
            "lavfi",
            "-i",
            &ffmpeg_source,
            "-c:v",
            "libx264",
            "-f",
            "rtp",
            "rtp://127.0.0.1:8004?rtcpport=8004",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    Ok(())
}
