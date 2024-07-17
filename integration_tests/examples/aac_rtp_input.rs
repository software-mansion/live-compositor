use anyhow::Result;
use live_compositor::types::Resolution;
use log::info;
use serde_json::json;
use std::time::Duration;

use integration_tests::{
    examples::{self, download_file, run_example},
    ffmpeg::{start_ffmpeg_receive, start_ffmpeg_send_audio, start_ffmpeg_send_video},
};

const BUNNY_FILE_URL: &str =
    "https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4";
const BUNNY_FILE_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/examples/assets/BigBuckBunny.mp4"
);
const VIDEO_RESOLUTION: Resolution = Resolution {
    width: 1280,
    height: 720,
};

const IP: &str = "127.0.0.1";
const INPUT_1_PORT: u16 = 8002;
const INPUT_2_PORT: u16 = 8004;
const OUTPUT_VIDEO_PORT: u16 = 8010;
const OUTPUT_AUDIO_PORT: u16 = 8012;

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
            "type": "rtp_stream",
            "port": INPUT_1_PORT,
            "video": {
                "decoder": "ffmpeg_h264"
            },
        }),
    )?;

    info!("[example] Download sample.");
    let bunny_path = download_file(BUNNY_FILE_URL, BUNNY_FILE_PATH)?;

    info!("[example] Send register input request.");
    examples::post(
        "input/input_2/register",
        &json!({
            "type": "rtp_stream",
            "port": INPUT_2_PORT,
            "audio": {
                "decoder": "aac",
                // both of these options can be acquired by passing the
                // `-sdp_file FILENAME` flag to the ffmpeg instance which will
                // stream data to the compositor.
                // ffmpeg will then write out an sdp file containing both fields.
                "rtp_mode": "high_bitrate",
                "audio_specific_config": "121056E500",
            },
        }),
    )?;

    info!("[example] Send register output request.");
    examples::post(
        "output/output_1/register",
        &json!({
            "type": "rtp_stream",
            "ip": IP,
            "port": OUTPUT_VIDEO_PORT,
            "video": {
                "resolution": {
                    "width": VIDEO_RESOLUTION.width,
                    "height": VIDEO_RESOLUTION.height,
                },
                "encoder": {
                    "type": "ffmpeg_h264",
                    "preset": "fast"
                },
                "initial": {
                    "root": {
                        "type": "input_stream",
                        "input_id": "input_1"
                    }
                },
                "resolution": { "width": VIDEO_RESOLUTION.width, "height": VIDEO_RESOLUTION.height },
            }
        }),
    )?;

    info!("[example] Send register output request.");
    examples::post(
        "output/output_2/register",
        &json!({
            "type": "rtp_stream",
            "ip": IP,
            "port": OUTPUT_AUDIO_PORT,
            "audio": {
                "initial": {
                    "inputs": [
                        {"input_id": "input_2"},
                    ]
                },
                "encoder": {
                    "type": "opus",
                    "channels": "stereo",
                }
            }
        }),
    )?;

    std::thread::sleep(Duration::from_millis(500));

    info!("[example] Start pipeline");
    examples::post("start", &json!({}))?;

    start_ffmpeg_send_video(IP, INPUT_1_PORT, bunny_path.clone())?;
    start_ffmpeg_send_audio(IP, INPUT_2_PORT, bunny_path, "aac")?;

    Ok(())
}
