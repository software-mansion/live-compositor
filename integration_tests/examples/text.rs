use anyhow::Result;
use compositor_api::types::Resolution;
use live_compositor::server;
use log::{error, info};
use serde_json::json;
use std::thread;

use integration_tests::examples::{self, start_ffplay, start_websocket_thread};

const VIDEO_RESOLUTION: Resolution = Resolution {
    width: 1920,
    height: 1080,
};

const IP: &str = "127.0.0.1";
const OUTPUT_PORT: u16 = 8002;

fn main() {
    ffmpeg_next::format::network::init();

    thread::spawn(|| {
        if let Err(err) = start_example_client_code() {
            error!("{err}")
        }
    });

    server::run()
}

fn start_example_client_code() -> Result<()> {
    info!("[example] Start listening on output port.");
    start_ffplay(IP, OUTPUT_PORT, None)?;
    start_websocket_thread();

    info!("[example] Send register output request.");
    examples::post(
        "output/output_1/register",
        &json!({
            "type": "rtp_stream",
            "ip": IP,
            "port": OUTPUT_PORT,
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
                    "root": {
                        "type": "text",
                        "text": "VideoCompositor🚀\nSecond Line\nLorem ipsum dolor sit amet consectetur adipisicing elit. Soluta delectus optio fugit maiores eaque ab totam, veritatis aperiam provident, aliquam consectetur deserunt cumque est? Saepe tenetur impedit culpa asperiores id?",
                        "font_size": 100.0,
                        "font_family": "Comic Sans MS",
                        "align": "center",
                        "wrap": "word",
                        "background_color_rgba": "#00800000",
                        "weight": "bold",
                        "width": VIDEO_RESOLUTION.width,
                        "height": VIDEO_RESOLUTION.height,
                    }
                }
            }
        }),
    )?;

    info!("[example] Start pipeline");
    examples::post("start", &json!({}))?;

    Ok(())
}
