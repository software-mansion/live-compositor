use anyhow::Result;
use compositor_api::types::Resolution;
use serde_json::json;
use std::{thread::sleep, time::Duration};

use integration_tests::examples::{self, run_example};

const VIDEO_RESOLUTION: Resolution = Resolution {
    width: 1280,
    height: 720,
};

fn main() {
    run_example(client_code);
}

fn client_code() -> Result<()> {
    examples::post(
        "input/input_1/register",
        &json!({
            "type": "whip",
            "video": {
              "decoder": "ffmpeg_h264"
            },
            "audio": {
              "decoder": "opus"
            },
        }),
    )?;

    examples::post(
        "input/input_2/register",
        &json!({
            "type": "whip",
            "video": {
              "decoder": "ffmpeg_h264"
            },
            "audio": {
              "decoder": "opus"
            },
        }),
    )?;

    examples::post(
        "output/output_1/register",
        &json!({
            "type": "whip",
            "endpoint_url": "https://g.webrtc.live-video.net:4443/v2/offer",
            "bearer_token": "", // your Bearer token
            "video": {
                "resolution": {
                    "width": VIDEO_RESOLUTION.width,
                    "height": VIDEO_RESOLUTION.height,
                },
                "encoder": {
                    "type": "ffmpeg_h264",
                    "preset": "medium"
                },
                "initial": {
                    "root": {
                        "type": "view",
                        "background_color_rgba": "#4d4d4dff",
                        "children": [
                            {
                                "type": "rescaler",
                                "child": { "type": "input_stream", "input_id": "input_1" }
                            },
                            {
                                "type": "rescaler",
                                "child": { "type": "input_stream", "input_id": "input_2" }
                            }
                        ]
                    }
                },

            },
            "audio": {
                "encoder": {
                    "type": "opus",
                    "channels": "stereo",
                },
                "initial": {
                    "inputs": [
                        {"input_id": "input_1"}
                    ]
                }
            }

        }),
    )?;

    std::thread::sleep(Duration::from_millis(500));

    examples::post("start", &json!({}))?;

    sleep(Duration::from_secs(300));
    examples::post("output/output_1/unregister", &json!({}))?;

    Ok(())
}
