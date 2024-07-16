use anyhow::Result;
use live_compositor::{server, types::Resolution};
use log::{error, info};
use serde_json::json;
use std::{
    process::{Command, Stdio},
    thread,
    time::Duration,
};

use integration_tests::utils::{self, start_websocket_thread};

const VIDEO_RESOLUTION: Resolution = Resolution {
    width: 1280,
    height: 720,
};

const IP: &str = "127.0.0.1";
const OUTPUT_VIDEO_PORT: u16 = 8002;

fn main() {
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
    thread::sleep(Duration::from_secs(2));
    start_websocket_thread();

    info!("[example] Send register input request.");
    utils::post(
        "input/input_1/register",
        &json!({
            "type": "decklink",
            "display_name": "DeckLink Quad HDMI Recorder (1)",
            "enable_audio": true,
        }),
    )?;

    info!("[example] Send register output video request.");
    utils::post(
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

    let gst_output_command =  [
        "gst-launch-1.0 -v ",
        "rtpptdemux name=demux ",
        &format!("tcpclientsrc host={IP} port={OUTPUT_VIDEO_PORT} ! \"application/x-rtp-stream\" ! rtpstreamdepay ! queue ! demux. "),
        "demux.src_96 ! \"application/x-rtp,media=video,clock-rate=90000,encoding-name=H264\" ! queue ! rtph264depay ! decodebin ! videoconvert ! autovideosink ",
        "demux.src_97 ! \"application/x-rtp,media=audio,clock-rate=48000,encoding-name=OPUS\" ! queue ! rtpopusdepay ! decodebin ! audioconvert ! autoaudiosink ",
    ].concat();
    Command::new("bash")
        .arg("-c")
        .arg(gst_output_command)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    std::thread::sleep(Duration::from_millis(1000));

    info!("[example] Start pipeline");
    utils::post("start", &json!({}))?;

    Ok(())
}
