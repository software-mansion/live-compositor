use anyhow::Result;
use live_compositor::types::Resolution;
use log::info;
use serde_json::json;
use std::time::Duration;

use integration_tests::{
    examples::{self, run_example},
    ffmpeg::start_ffmpeg_receive,
};

const BUNNY_URL: &str =
    "https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4";

const VIDEO_RESOLUTION: Resolution = Resolution {
    width: 1280,
    height: 720,
};

const IP: &str = "127.0.0.1";
const OUTPUT_VIDEO_PORT: u16 = 8002;
const OUTPUT_AUDIO_PORT: u16 = 8004;

fn main() {
    run_example(client_code);
}

fn client_code() -> Result<()> {
    info!("[example] Start listening on output port.");
    start_ffmpeg_receive(Some(OUTPUT_VIDEO_PORT), Some(OUTPUT_AUDIO_PORT))?;

    info!("[example] Send register input request.");
    examples::post(
        "input/input_1/register",
        &json!({
            "type": "mp4",
            "url": BUNNY_URL
        }),
    )?;

    let shader_source = include_str!("./silly.wgsl");
    info!("[example] Register shader transform");
    examples::post(
        "shader/shader_example_1/register",
        &json!({
            "source": shader_source,
        }),
    )?;

    info!("[example] Send register output video request.");
    examples::post(
        "output/output_1/register",
        &json!({
            "type": "rtp_stream",
            "port": OUTPUT_VIDEO_PORT,
            "ip": IP,
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
                        "id": "input_1",
                        "type": "input_stream",
                        "input_id": "input_1",
                    }
                }
            }
        }),
    )?;

    info!("[example] Send register output audio request.");
    examples::post(
        "output/output_2/register",
        &json!({
            "type": "rtp_stream",
            "port": OUTPUT_AUDIO_PORT,
            "ip": IP,
            "audio": {
                "initial": {
                    "inputs": [
                        {"input_id": "input_1"}
                    ]
                },
                "encoder": {
                    "type": "opus",
                    "channels": "stereo"
                }
            }
        }),
    )?;

    std::thread::sleep(Duration::from_millis(500));

    info!("[example] Start pipeline");
    examples::post("start", &json!({}))?;

    Ok(())
}
