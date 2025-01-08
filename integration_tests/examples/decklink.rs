use anyhow::Result;
use compositor_api::types::Resolution;
use serde_json::json;
use std::time::Duration;

use integration_tests::{
    examples::{self, run_example},
    gstreamer::start_gst_receive_tcp,
};

const VIDEO_RESOLUTION: Resolution = Resolution {
    width: 1280,
    height: 720,
};

const IP: &str = "127.0.0.1";
const OUTPUT_VIDEO_PORT: u16 = 8002;

fn main() {
    run_example(client_code);
}

fn client_code() -> Result<()> {
    examples::post(
        "input/input_1/register",
        &json!({
            "type": "decklink",
            "display_name": "DeckLink Quad HDMI Recorder (1)",
            "enable_audio": true,
        }),
    )?;

    examples::post(
        "output/output_1/register",
        &json!({
            "type": "rtp_stream",
            "transport_protocol": "tcp_server",
            "port": OUTPUT_VIDEO_PORT,
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
                        "type": "view",
                        "background_color_rgba": "#4d4d4dff",
                        "children": [
                            {
                                "type": "rescaler",
                                "top": 10,
                                "left": 10,
                                "width": VIDEO_RESOLUTION.width - 20,
                                "height": VIDEO_RESOLUTION.height - 20,
                                "child": {
                                    "id": "input_1",
                                    "type": "input_stream",
                                    "input_id": "input_1",
                                }
                            }
                        ]
                    }
                }
            },
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

    start_gst_receive_tcp(IP, OUTPUT_VIDEO_PORT, true, false)?;

    std::thread::sleep(Duration::from_millis(1000));

    examples::post("start", &json!({}))?;

    Ok(())
}
