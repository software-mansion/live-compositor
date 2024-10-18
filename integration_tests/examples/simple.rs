use anyhow::Result;
use compositor_api::types::Resolution;
use crossbeam_channel::Sender;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::time::Duration;
use tokio_tungstenite::tungstenite;
use tracing::{error, info};

use integration_tests::{
    examples::{self, run_example, TestSample},
    ffmpeg::{start_ffmpeg_receive, start_ffmpeg_send},
};

const VIDEO_RESOLUTION: Resolution = Resolution {
    width: 1280,
    height: 720,
};

const IP: &str = "127.0.0.1";
const INPUT_PORT: u16 = 8002;
const OUTPUT_PORT: u16 = 8004;

fn main() {
    run_example(client_code);
}

fn client_code() -> Result<()> {
    start_ffmpeg_receive(Some(OUTPUT_PORT), None)?;
    let (msg_sender, msg_receiver) = crossbeam_channel::unbounded();
    start_server_msg_listener(8081, msg_sender);

    examples::post(
        "input/input_1/register",
        &json!({
            "type": "mp4",
            "url": "https://filesamples.com/samples/video/mp4/sample_1280x720.mp4",
            "required": true,
            "offset_ms": 0,
        }),
    )?;

    let shader_source = include_str!("./silly.wgsl");
    examples::post(
        "shader/shader_example_1/register",
        &json!({
            "source": shader_source,
        }),
    )?;

    examples::post(
        "output/output_1/register",
        &json!({
            "type": "rtp_stream",
            "port": OUTPUT_PORT,
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
                },
                "send_eos_when": {
                    "any_input": true,
                }
            }
        }),
    )?;

    std::thread::sleep(Duration::from_millis(500));

    examples::post("start", &json!({}))?;

    // start_ffmpeg_send(IP, Some(INPUT_PORT), None, TestSample::Sample)?;
    for msg in msg_receiver.iter() {
        error!("MSG: {msg:?}");
        if let tungstenite::Message::Text(msg) = msg {
            if msg.contains("\"type\":\"OUTPUT_DONE\",\"output_id\":\"output_1\"") {
                info!("breaking");
                break;
            }
        }
    }

    examples::post(
        "output/output_1/update",
        &json!({
            "video": {
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
        }),
    )?;

    Ok(())
}

fn start_server_msg_listener(port: u16, event_sender: Sender<tungstenite::Message>) {
    std::thread::Builder::new()
        .name("Websocket Thread".to_string())
        .spawn(move || {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async { server_msg_listener(port, event_sender).await });
        })
        .unwrap();
}

async fn server_msg_listener(port: u16, event_sender: Sender<tungstenite::Message>) {
    let url = format!("ws://127.0.0.1:{}/ws", port);

    let (ws_stream, _) = tokio_tungstenite::connect_async(url)
        .await
        .expect("Failed to connect");

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let (mut outgoing, mut incoming) = ws_stream.split();

    let sender_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let tungstenite::Message::Close(None) = &msg {
                let _ = outgoing.send(msg).await;
                return;
            }
            match outgoing.send(msg).await {
                Ok(()) => (),
                Err(e) => {
                    println!("Send Loop: {:?}", e);
                    let _ = outgoing.send(tungstenite::Message::Close(None)).await;
                    return;
                }
            }
        }
    });

    let receiver_task = tokio::spawn(async move {
        while let Some(result) = incoming.next().await {
            match result {
                Ok(tungstenite::Message::Close(_)) => {
                    let _ = tx.send(tungstenite::Message::Close(None));
                    return;
                }
                Ok(tungstenite::Message::Ping(data)) => {
                    if tx.send(tungstenite::Message::Pong(data)).is_err() {
                        return;
                    }
                }
                Err(_) => {
                    let _ = tx.send(tungstenite::Message::Close(None));
                    return;
                }
                Ok(msg) => {
                    event_sender.send(msg).unwrap();
                }
            }
        }
    });

    sender_task.await.unwrap();
    receiver_task.await.unwrap();
}
