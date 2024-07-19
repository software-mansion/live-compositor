use anyhow::Result;
use compositor_api::types::Resolution;
use serde_json::json;
use std::{thread::sleep, time::Duration};

use integration_tests::{
    examples::{self, get_client_ip, run_example, TestSample},
    ffmpeg::{start_ffmpeg_receive, start_ffmpeg_send},
    gstreamer::{start_gst_receive_tcp, start_gst_receive_udp, start_gst_send_udp},
};

const VIDEO_RESOLUTION: Resolution = Resolution {
    width: 1280,
    height: 720,
};

const IP: &str = "127.0.0.1";
const INPUT_1_PORT: u16 = 8000;
// const INPUT_2_PORT: u16 = 8504;
// const INPUT_3_PORT: u16 = 8506;
// const INPUT_4_PORT: u16 = 8508;
// const INPUT_5_PORT: u16 = 8510;
// const INPUT_6_PORT: u16 = 8512;
// const INPUT_7_PORT: u16 = 8514;
const OUTPUT_VIDEO_PORT: u16 = 8002;
// const OUTPUT_AUDIO_PORT: u16 = 8518;

fn main() {
    run_example(start_example_client_code);
}

fn start_example_client_code() -> Result<()> {
    // start_ffmpeg_receive(Some(OUTPUT_VIDEO_PORT), None)?;

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

    // examples::post(
    //     "input/input_2/register",
    //     &json!({
    //         "type": "rtp_stream",
    //         "port": INPUT_2_PORT,
    //         "audio": {
    //             "decoder": "opus"
    //         },
    //     }),
    // )?;

    // examples::post(
    //     "input/input_3/register",
    //     &json!({
    //         "type": "rtp_stream",
    //         "port": INPUT_3_PORT,
    //         "video": {
    //             "decoder": "ffmpeg_h264"
    //         },
    //     }),
    // )?;

    // examples::post(
    //     "input/input_4/register",
    //     &json!({
    //         "type": "rtp_stream",
    //         "port": INPUT_4_PORT,
    //         "audio": {
    //             "decoder": "opus"
    //         },
    //     }),
    // )?;

    // examples::post(
    //     "input/input_5/register",
    //     &json!({
    //         "type": "rtp_stream",
    //         "port": INPUT_5_PORT,
    //         "video": {
    //             "decoder": "ffmpeg_h264"
    //         },
    //     }),
    // )?;

    // examples::post(
    //     "input/input_6/register",
    //     &json!({
    //         "type": "rtp_stream",
    //         "port": INPUT_6_PORT,
    //         "video": {
    //             "decoder": "ffmpeg_h264"
    //         },
    //     }),
    // )?;

    // examples::post(
    //     "input/input_7/register",
    //     &json!({
    //         "type": "rtp_stream",
    //         "port": INPUT_7_PORT,
    //         "video": {
    //             "decoder": "ffmpeg_h264"
    //         },
    //     }),
    // )?;

    examples::post(
        "output/output_1/register",
        &json!({
            "type": "rtp_stream",
            "ip": get_client_ip().unwrap(),
            // "transport_protocol": "tcp_server",
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
                //     "root": {
                //         "type": "tiles",
                //         "children": [
                //             {
                //                 "type": "input_stream",
                //                 "input_id": "input_1"
                //             },
                //             {
                //                 "type": "input_stream",
                //                 "input_id": "input_3"
                //             },
                //             {
                //                 "type": "input_stream",
                //                 "input_id": "input_5"
                //             },
                //             {
                //                 "type": "input_stream",
                //                 "input_id": "input_6"
                //             },
                //             {
                //                 "type": "input_stream",
                //                 "input_id": "input_7"
                //             }
                //         ]
                //     }
                },
                "resolution": { "width": VIDEO_RESOLUTION.width, "height": VIDEO_RESOLUTION.height },
            },
        }),
    )?;

    // examples::post(
    //     "output/output_2/register",
    //     &json!({
    //         "type": "rtp_stream",
    //         "ip": get_client_ip().unwrap(),
    //         "port": OUTPUT_AUDIO_PORT,
    //         "audio": {
    //             "initial": {
    //                 "inputs": [
    //                     {"input_id": "input_2"},
    //                     {"input_id": "input_4"}
    //                 ]
    //             },
    //             "encoder": {
    //                 "type": "opus",
    //                 "channels": "stereo",
    //             }
    //         },
    //     }),
    // )?;
    std::thread::sleep(Duration::from_millis(500));

    examples::post("start", &json!({}))?;

    // start_gst_send_udp(
    //     Some(INPUT_1_PORT),
    //     // Some(INPUT_2_PORT),
    //     None,
    //     TestSample::TestPattern,
    // )?;

    start_ffmpeg_send(Some(INPUT_1_PORT), None, TestSample::TestPattern)?;
    // start_ffmpeg_send(
    //     Some(INPUT_3_PORT),
    //     Some(INPUT_4_PORT),
    //     TestSample::ElephantsDream,
    // )?;
    // start_gst_send_udp(Some(INPUT_5_PORT), None, TestSample::Sample)?;
    // start_ffmpeg_send(Some(INPUT_6_PORT), None, TestSample::SampleLoop)?;
    // start_ffmpeg_send(Some(INPUT_7_PORT), None, TestSample::TestPattern)?;

    Ok(())
}
