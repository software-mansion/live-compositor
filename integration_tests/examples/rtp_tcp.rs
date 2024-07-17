use anyhow::Result;
use compositor_api::types::Resolution;
use live_compositor::server;
use log::{error, info};
use serde_json::json;
use std::{
    process::{Command, Stdio},
    thread,
    time::Duration,
};

use integration_tests::examples::{self, download_file, start_websocket_thread};

const SAMPLE_FILE_URL: &str =
    "https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4";
const SAMPLE_FILE_PATH: &str = "examples/assets/BigBuckBunny.mp4";
const VIDEO_RESOLUTION: Resolution = Resolution {
    width: 1280,
    height: 720,
};

const IP: &str = "127.0.0.1";
const INPUT_1_PORT: u16 = 8002;
const INPUT_2_PORT: u16 = 8004;
const OUTPUT_PORT: u16 = 8006;

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
    info!("[example] Download sample.");
    let sample_path = download_file(SAMPLE_FILE_URL, SAMPLE_FILE_PATH)?;
    thread::sleep(Duration::from_secs(2));
    start_websocket_thread();

    info!("[example] Send register input request.");
    examples::post(
        "input/input_1/register",
        &json!({
            "type": "rtp_stream",
            "transport_protocol": "tcp_server",
            "port": INPUT_1_PORT,
            "video": {
                "decoder": "ffmpeg_h264"
            },
        }),
    )?;

    examples::post(
        "input/input_2/register",
        &json!({
            "type": "rtp_stream",
            "transport_protocol": "tcp_server",
            "port": INPUT_2_PORT,
            "audio": {
                "decoder": "opus"
            },
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

    info!("[example] Send register output request.");
    examples::post(
        "output/output_1/register",
        &json!({
            "type": "rtp_stream",
            "transport_protocol": "tcp_server",
            "port": OUTPUT_PORT,
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
            },
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
                        "type": "shader",
                        "id": "shader_node_1",
                        "shader_id": "shader_example_1",
                        "children": [
                            {
                                "id": "input_1",
                                "type": "input_stream",
                                "input_id": "input_1",
                            }
                        ],
                        "resolution": { "width": VIDEO_RESOLUTION.width, "height": VIDEO_RESOLUTION.height },
                    }
                }
            }
        }),
    )?;
    let gst_output_command =  [
        "gst-launch-1.0 -v ",
        "rtpptdemux name=demux ",
        &format!("tcpclientsrc host={IP} port={OUTPUT_PORT} ! \"application/x-rtp-stream\" ! rtpstreamdepay ! queue ! demux. "),
        "demux.src_96 ! \"application/x-rtp,media=video,clock-rate=90000,encoding-name=H264\" ! queue ! rtph264depay ! decodebin ! videoconvert ! autovideosink ",
        "demux.src_97 ! \"application/x-rtp,media=audio,clock-rate=48000,encoding-name=OPUS\" ! queue ! rtpopusdepay ! decodebin ! audioconvert ! autoaudiosink ",
    ].concat();

    Command::new("bash")
        .arg("-c")
        .arg(gst_output_command)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    std::thread::sleep(Duration::from_millis(500));

    info!("[example] Start pipeline");
    examples::post("start", &json!({}))?;

    let sample_path_str = sample_path.to_string_lossy().to_string();

    let gst_input_command = [
        "gst-launch-1.0 -v ",
        &format!("filesrc location={sample_path_str} ! qtdemux name=demux "),
        &format!("demux.video_0 ! queue ! h264parse ! rtph264pay config-interval=1 !  application/x-rtp,payload=96  ! rtpstreampay ! tcpclientsink host={IP} port={INPUT_1_PORT} "),
        &format!("demux.audio_0 ! queue ! decodebin ! audioconvert ! audioresample ! opusenc ! rtpopuspay ! application/x-rtp,payload=97 !  rtpstreampay ! tcpclientsink host={IP} port={INPUT_2_PORT} "),
    ].concat();

    Command::new("bash")
        .arg("-c")
        .arg(gst_input_command)
        .spawn()?;

    Ok(())
}
