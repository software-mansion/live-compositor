use anyhow::Result;
use live_compositor::{server, types::Resolution};
use log::{error, info};
use serde_json::json;
use std::{
    env,
    thread::{self},
    time::Duration,
};

use integration_tests::examples::{
    self, ff_stream_sample, start_ffplay, start_websocket_thread, TestSample,
};

const VIDEO_RESOLUTION: Resolution = Resolution {
    width: 1280,
    height: 720,
};

const IP: &str = "127.0.0.1";
const INPUT_1_PORT: u16 = 8002;
const INPUT_2_PORT: u16 = 8004;
const INPUT_3_PORT: u16 = 8006;
const INPUT_4_PORT: u16 = 8008;
const INPUT_5_PORT: u16 = 8014;
const INPUT_6_PORT: u16 = 8016;
const INPUT_7_PORT: u16 = 8018;
const OUTPUT_VIDEO_PORT: u16 = 8010;
const OUTPUT_AUDIO_PORT: u16 = 8012;

fn main() {
    env::set_var("LIVE_COMPOSITOR_WEB_RENDERER_ENABLE", "0");
    ffmpeg_next::format::network::init();

    thread::spawn(|| {
        if let Err(err) = start_example_client_code() {
            error!("{err}")
        }
    });

    server::run();
}

fn start_example_client_code() -> Result<()> {
    info!("[example] Start listening on output port.");
    start_ffplay(IP, Some(OUTPUT_VIDEO_PORT), Some(OUTPUT_AUDIO_PORT))?;
    start_websocket_thread();

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

    info!("[example] Send register input request.");
    examples::post(
        "input/input_2/register",
        &json!({
            "type": "rtp_stream",
            "port": INPUT_2_PORT,
            "audio": {
                "decoder": "opus"
            },
        }),
    )?;

    info!("[example] Send register input request.");
    examples::post(
        "input/input_3/register",
        &json!({
            "type": "rtp_stream",
            "port": INPUT_3_PORT,
            "video": {
                "decoder": "ffmpeg_h264"
            },
        }),
    )?;

    info!("[example] Send register input request.");
    examples::post(
        "input/input_4/register",
        &json!({
            "type": "rtp_stream",
            "port": INPUT_4_PORT,
            "audio": {
                "decoder": "opus"
            },
        }),
    )?;

    info!("[example] Send register input request.");
    examples::post(
        "input/input_5/register",
        &json!({
            "type": "rtp_stream",
            "port": INPUT_5_PORT,
            "video": {
                "decoder": "ffmpeg_h264"
            },
        }),
    )?;

    info!("[example] Send register input request.");
    examples::post(
        "input/input_6/register",
        &json!({
            "type": "rtp_stream",
            "port": INPUT_6_PORT,
            "video": {
                "decoder": "ffmpeg_h264"
            },
        }),
    )?;

    info!("[example] Send register input request.");
    examples::post(
        "input/input_7/register",
        &json!({
            "type": "rtp_stream",
            "port": INPUT_7_PORT,
            "video": {
                "decoder": "ffmpeg_h264"
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
                        "type": "tiles",
                        "children": [
                            {
                                "type": "input_stream",
                                "input_id": "input_1"
                            },
                            {
                                "type": "input_stream",
                                "input_id": "input_3"
                            },
                            {
                                "type": "input_stream",
                                "input_id": "input_5"
                            },
                            {
                                "type": "input_stream",
                                "input_id": "input_6"
                            },
                            {
                                "type": "input_stream",
                                "input_id": "input_7"
                            }
                        ]
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
                        {"input_id": "input_4"}
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

    ff_stream_sample(
        IP,
        Some(INPUT_1_PORT),
        Some(INPUT_2_PORT),
        TestSample::BigBuckBunny,
    )?;
    ff_stream_sample(
        IP,
        Some(INPUT_3_PORT),
        Some(INPUT_4_PORT),
        TestSample::ElephantsDream,
    )?;
    ff_stream_sample(IP, Some(INPUT_5_PORT), None, TestSample::Sample)?;
    ff_stream_sample(IP, Some(INPUT_6_PORT), None, TestSample::SampleLoop)?;
    ff_stream_sample(IP, Some(INPUT_7_PORT), None, TestSample::Generic)?;

    Ok(())
}
