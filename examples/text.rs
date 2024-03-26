use anyhow::Result;
use log::{error, info};
use serde_json::json;
use std::{
    env,
    process::{Command, Stdio},
    thread,
    time::Duration,
};
use video_compositor::{logger, server, types::Resolution};

use crate::common::{start_websocket_thread, write_video_example_sdp_file};

#[path = "./common/common.rs"]
mod common;

const VIDEO_RESOLUTION: Resolution = Resolution {
    width: 1920,
    height: 1080,
};

fn main() {
    env::set_var("LIVE_COMPOSITOR_WEB_RENDERER_ENABLE", "0");
    ffmpeg_next::format::network::init();
    logger::init_logger();

    thread::spawn(|| {
        if let Err(err) = start_example_client_code() {
            error!("{err}")
        }
    });

    server::run()
}

fn start_example_client_code() -> Result<()> {
    info!("[example] Start listening on output port.");
    let output_sdp = write_video_example_sdp_file("127.0.0.1", 8002)?;
    Command::new("ffplay")
        .args(["-protocol_whitelist", "file,rtp,udp", &output_sdp])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    thread::sleep(Duration::from_secs(2));
    start_websocket_thread();

    info!("[example] Send register output request.");
    common::post(&json!({
        "type": "register",
        "entity_type": "output_stream",
        "output_id": "output_1",
        "port": 8002,
        "ip": "127.0.0.1",
        "video": {
            "resolution": {
                "width": VIDEO_RESOLUTION.width,
                "height": VIDEO_RESOLUTION.height,
            },
            "encoder_preset": "ultrafast",
            "initial": {
                "type": "text",
                "text": "VideoCompositor🚀\nSecond Line\nLorem ipsum dolor sit amet consectetur adipisicing elit. Soluta delectus optio fugit maiores eaque ab totam, veritatis aperiam provident, aliquam consectetur deserunt cumque est? Saepe tenetur impedit culpa asperiores id?",
                "font_size": 100.0,
                "font_family": "Comic Sans MS",
                "align": "center",
                "wrap": "word",
                "background_color_rgba": "#00800000",
                "weight": "bold",
                "width": 1920,
                "height": 1080,
            }
        }
    }))?;

    info!("[example] Start pipeline");
    common::post(&json!({
        "type": "start",
    }))?;

    Ok(())
}
