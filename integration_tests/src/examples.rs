use anyhow::{anyhow, Result};

use futures_util::{SinkExt, StreamExt};
use live_compositor::{config::read_config, types::Resolution};
use log::error;
use reqwest::{blocking::Response, StatusCode};
use std::{
    env,
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::Duration,
};
use tokio_tungstenite::tungstenite;
use tracing::info;

use serde::Serialize;

pub fn post<T: Serialize + ?Sized>(route: &str, json: &T) -> Result<Response> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .post(format!(
            "http://127.0.0.1:{}/api/{}",
            read_config().api_port,
            route
        ))
        .timeout(Duration::from_secs(100))
        .json(json)
        .send()
        .unwrap();
    if response.status() >= StatusCode::BAD_REQUEST {
        log_request_error(&json, response);
        return Err(anyhow!("request failed"));
    }
    Ok(response)
}

pub fn start_websocket_thread() {
    thread::Builder::new()
        .name("Websocket Thread".to_string())
        .spawn(|| {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async { websocket_thread().await });
        })
        .unwrap();
}

async fn websocket_thread() {
    let url = format!("ws://127.0.0.1:{}/ws", read_config().api_port);

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
                _ => {
                    info!("Received compositor event: {:?}", result);
                }
            }
        }
    });

    sender_task.await.unwrap();
    receiver_task.await.unwrap();
}

pub fn download_file(url: &str, path: &str) -> Result<PathBuf> {
    let sample_path = env::current_dir()?.join(path);
    fs::create_dir_all(sample_path.parent().unwrap())?;

    if sample_path.exists() {
        return Ok(sample_path);
    }

    let mut resp = reqwest::blocking::get(url)?;
    let mut out = File::create(sample_path.clone())?;
    io::copy(&mut resp, &mut out)?;
    Ok(sample_path)
}

// plays H264 encoded video and Opus encoded audio using ffmpeg. If no port given, returns Error
pub fn start_ffplay(ip: &str, video_port: Option<u16>, audio_port: Option<u16>) -> Result<()> {
    let output_sdp_path = match video_port {
        Some(video_port) => {
            if let Some(audio_port) = audio_port {
                write_video_audio_example_sdp_file(ip, video_port, audio_port)
            } else {
                write_video_example_sdp_file(ip, video_port)
            }
        }
        None => {
            if let Some(audio_port) = audio_port {
                write_audio_example_sdp_file(ip, audio_port)
            } else {
                return Err(anyhow!("no port given"));
            }
        }
    }?;

    Command::new("ffplay")
        .args(["-protocol_whitelist", "file,rtp,udp", &output_sdp_path])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    thread::sleep(Duration::from_secs(2));

    Ok(())
}

// plays H264 encoded video and Opus encoded audio using gstreamer. If no port given, returns Error
pub fn start_gstplay(ip: &str, port: u16, video: bool, audio: bool) -> Result<()> {
    let mut gst_output_command = [
        "gst-launch-1.0 -v ",
        "rtpptdemux name=demux ",
        &format!("tcpclientsrc host={} port={} ! \"application/x-rtp-stream\" ! rtpstreamdepay ! queue ! demux. ", ip, port)
        ].concat();

    if video {
        gst_output_command.push_str("demux.src_96 ! \"application/x-rtp,media=video,clock-rate=90000,encoding-name=H264\" ! queue ! rtph264depay ! decodebin ! videoconvert ! autovideosink ");
    }
    if audio {
        gst_output_command.push_str("demux.src_97 ! \"application/x-rtp,media=audio,clock-rate=48000,encoding-name=OPUS\" ! queue ! rtpopusdepay ! decodebin ! audioconvert ! autoaudiosink ");
    }

    Command::new("bash")
        .arg("-c")
        .arg(gst_output_command)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    thread::sleep(Duration::from_secs(2));

    Ok(())
}

// GStreamer will stream with H264 video encoding and Opus audio encoding if video and audio ports will be given. Will return Error otherwise.
pub fn gst_stream(
    ip: &str,
    video_port: Option<u16>,
    audio_port: Option<u16>,
    path: &str,
) -> Result<()> {
    let mut gst_input_command = [
        "gst-launch-1.0 -v ",
        &format!("filesrc location={path} ! qtdemux name=demux "),
    ]
    .concat();

    let mut any_port_available = false;
    if let Some(port) = video_port {
        any_port_available = true;
        gst_input_command = gst_input_command + &format!("demux.video_0 ! queue ! h264parse ! rtph264pay config-interval=1 !  application/x-rtp,payload=96  ! rtpstreampay ! tcpclientsink host={ip} port={port} ");
    }
    if let Some(port) = audio_port {
        any_port_available = true;
        gst_input_command = gst_input_command + &format!("demux.audio_0 ! queue ! decodebin ! audioconvert ! audioresample ! opusenc ! rtpopuspay ! application/x-rtp,payload=97 !  rtpstreampay ! tcpclientsink host={ip} port={port} ");
    }

    if !any_port_available {
        return Err(anyhow!("no port given"));
    }

    Command::new("bash")
        .arg("-c")
        .arg(gst_input_command)
        .spawn()?;

    Ok(())
}

// GStreamer will stream testsrc with H264 video encoding and Opus audio encoding if video and audio ports will be given. Will return Error otherwise.
pub fn gst_stream_testsrc(
    ip: &str,
    video_port: Option<u16>,
    audio_port: Option<u16>,
) -> Result<()> {
    let mut gst_input_command = [
        "gst-launch-1.0 -v videotestsrc ! ",
        "\"video/x-raw,format=I420,width=1920,height=1080,framerate=60/1\" ! ",
    ]
    .concat();

    let mut any_port_available = false;
    if let Some(port) = video_port {
        any_port_available = true;
        gst_input_command = gst_input_command + &format!(" x264enc tune=zerolatency speed-preset=superfast ! rtph264pay ! application/x-rtp,payload=96 ! rtpstreampay ! tcpclientsink host={ip} port={port}");
    }
    if let Some(port) = audio_port {
        any_port_available = true;
        gst_input_command = gst_input_command + &format!(" audiotestsrc ! audioconvert ! audioresample ! opusenc ! rtpopuspay ! application/x-rtp,payload=97 ! rtpstreampay ! tcpclientsink host={ip} port={port}");
    }

    if !any_port_available {
        return Err(anyhow!("no port given"));
    }

    Command::new("bash")
        .arg("-c")
        .arg(gst_input_command)
        .spawn()?;

    Ok(())
}

pub fn ff_stream_video_loop(ip: &str, port: u16, path: PathBuf) -> Result<()> {
    Command::new("ffmpeg")
        .args(["-stream_loop", "-1", "-re", "-i"])
        .arg(path)
        .args([
            "-an",
            "-c:v",
            "copy",
            "-f",
            "rtp",
            "-bsf:v",
            "h264_mp4toannexb",
            &format!("rtp://{ip}:{port}?rtcpport={port}"),
        ])
        .spawn()?;

    Ok(())
}

pub fn ff_stream(
    ip: &str,
    video_port: Option<u16>,
    audio_port: Option<u16>,
    path: PathBuf,
) -> Result<()> {
    let mut any_port_available = false;
    if let Some(port) = video_port {
        any_port_available = true;
        ff_stream_video(ip, port, path.clone())?;
    }
    if let Some(port) = audio_port {
        any_port_available = true;
        ff_stream_audio(ip, port, path, "libopus")?
    }

    if !any_port_available {
        return Err(anyhow!("no port given"));
    }

    Ok(())
}

pub fn ff_stream_video(ip: &str, port: u16, path: PathBuf) -> Result<()> {
    Command::new("ffmpeg")
        .args(["-re", "-i"])
        .arg(path)
        .args([
            "-an",
            "-c:v",
            "copy",
            "-f",
            "rtp",
            "-bsf:v",
            "h264_mp4toannexb",
            &format!("rtp://{ip}:{port}?rtcpport={port}"),
        ])
        .spawn()?;

    Ok(())
}

pub fn ff_stream_audio(ip: &str, port: u16, path: PathBuf, codec: &str) -> Result<()> {
    Command::new("ffmpeg")
        .args(["-stream_loop", "-1", "-re", "-i"])
        .arg(path.clone())
        .args([
            "-vn",
            "-c:a",
            codec,
            "-f",
            "rtp",
            &format!("rtp://{ip}:{port}?rtcpport={port}"),
        ])
        .spawn()?;

    Ok(())
}

pub fn ff_stream_testsrc(ip: &str, port: u16, resolution: Resolution) -> Result<()> {
    let ffmpeg_source = format!(
        "testsrc=s={}x{}:r=30,format=yuv420p",
        resolution.width, resolution.height
    );

    Command::new("ffmpeg")
        .args([
            "-re",
            "-f",
            "lavfi",
            "-i",
            &ffmpeg_source,
            "-c:v",
            "libx264",
            "-f",
            "rtp",
            &format!("rtp://{ip}:{port}?rtcpport={port}"),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    Ok(())
}

/// The SDP file will describe an RTP session on localhost with H264 encoding.
fn write_video_example_sdp_file(ip: &str, port: u16) -> Result<String> {
    let sdp_filepath = PathBuf::from(format!("/tmp/example_sdp_video_input_{}.sdp", port));
    let mut file = File::create(&sdp_filepath)?;
    file.write_all(
        format!(
            "\
                    v=0\n\
                    o=- 0 0 IN IP4 {}\n\
                    s=No Name\n\
                    c=IN IP4 {}\n\
                    m=video {} RTP/AVP 96\n\
                    a=rtpmap:96 H264/90000\n\
                    a=fmtp:96 packetization-mode=1\n\
                    a=rtcp-mux\n\
                ",
            ip, ip, port
        )
        .as_bytes(),
    )?;
    Ok(String::from(
        sdp_filepath
            .to_str()
            .ok_or_else(|| anyhow!("invalid utf string"))?,
    ))
}

/// The SDP file will describe an RTP session on localhost with H264 video encoding and Opus audio encoding.
fn write_video_audio_example_sdp_file(
    ip: &str,
    video_port: u16,
    audio_port: u16,
) -> Result<String> {
    let sdp_filepath = PathBuf::from(format!(
        "/tmp/example_sdp_video_audio_input_{}.sdp",
        video_port
    ));
    let mut file = File::create(&sdp_filepath)?;
    file.write_all(
        format!(
            "\
                    v=0\n\
                    o=- 0 0 IN IP4 {}\n\
                    s=No Name\n\
                    c=IN IP4 {}\n\
                    m=video {} RTP/AVP 96\n\
                    a=rtpmap:96 H264/90000\n\
                    a=fmtp:96 packetization-mode=1\n\
                    a=rtcp-mux\n\
                    m=audio {} RTP/AVP 97\n\
                    a=rtpmap:97 opus/48000/2\n\
                ",
            ip, ip, video_port, audio_port
        )
        .as_bytes(),
    )?;
    Ok(String::from(
        sdp_filepath
            .to_str()
            .ok_or_else(|| anyhow!("invalid utf string"))?,
    ))
}

/// The SDP file will describe an RTP session on localhost with  Opus audio encoding.
fn write_audio_example_sdp_file(ip: &str, port: u16) -> Result<String> {
    let sdp_filepath = PathBuf::from(format!("/tmp/example_sdp_audio_input_{}.sdp", port));
    let mut file = File::create(&sdp_filepath)?;
    file.write_all(
        format!(
            "\
                    v=0\n\
                    o=- 0 0 IN IP4 {}\n\
                    s=No Name\n\
                    c=IN IP4 {}\n\
                    m=audio {} RTP/AVP 97\n\
                    a=rtpmap:97 opus/48000/2\n\
                ",
            ip, ip, port
        )
        .as_bytes(),
    )?;
    Ok(String::from(
        sdp_filepath
            .to_str()
            .ok_or_else(|| anyhow!("invalid utf string"))?,
    ))
}

fn log_request_error<T: Serialize + ?Sized>(request_body: &T, response: Response) {
    let status = response.status();
    let request_str = serde_json::to_string_pretty(request_body).unwrap();
    let body_str = response.text().unwrap();

    let formated_body = get_formated_body(&body_str);
    error!(
        "Request failed:\n\nRequest: {}\n\nResponse code: {}\n\nResponse body:\n{}",
        request_str, status, formated_body
    )
}

fn get_formated_body(body_str: &str) -> String {
    let Ok(mut body_json) = serde_json::from_str::<serde_json::Value>(body_str) else {
        return body_str.to_string();
    };

    let Some(stack_value) = body_json.get("stack") else {
        return serde_json::to_string_pretty(&body_json).unwrap();
    };

    let errors: Vec<&str> = stack_value
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect();
    let msg_string = " - ".to_string() + &errors.join("\n - ");
    let body_map = body_json.as_object_mut().unwrap();
    body_map.remove("stack");
    format!(
        "{}\n\nError stack:\n{}",
        serde_json::to_string_pretty(&body_map).unwrap(),
        msg_string,
    )
}

pub enum TestSample {
    BigBuckBunny,
    ElephantsDream,
    Sample,
    SampleLoop,
    Generic,
}

pub enum TestCodec {
    AAC,
    LIBOPUS,
}

pub fn ff_stream_sample(
    ip: &str,
    video_port: Option<u16>,
    audio_port: Option<u16>,
    test_sample: TestSample,
) -> Result<()> {
    match test_sample {
        TestSample::BigBuckBunny => ff_stream(
            ip,
            video_port,
            audio_port,
            get_asset_path(test_sample).unwrap(),
        ),
        TestSample::ElephantsDream => ff_stream(
            ip,
            video_port,
            audio_port,
            get_asset_path(test_sample).unwrap(),
        ),
        TestSample::Sample => {
            if let Some(port) = video_port {
                ff_stream_video(ip, port, get_asset_path(test_sample).unwrap())
            } else {
                Err(anyhow!("video port required for test sample"))
            }
        }
        TestSample::SampleLoop => {
            if let Some(port) = video_port {
                ff_stream_video_loop(ip, port, get_asset_path(test_sample).unwrap())
            } else {
                Err(anyhow!("video port required for test sample"))
            }
        }
        TestSample::Generic => {
            if let Some(port) = video_port {
                ff_stream_testsrc(
                    ip,
                    port,
                    Resolution {
                        width: 1920,
                        height: 1080,
                    },
                )
            } else {
                Err(anyhow!("video port required for generic"))
            }
        }
    }
}

// receives and plays h264 encoded video and opus encoded audio
pub fn gst_stream_sample(
    ip: &str,
    video_port: Option<u16>,
    audio_port: Option<u16>,
    test_sample: TestSample,
) -> Result<()> {
    match test_sample {
        TestSample::BigBuckBunny => gst_stream(
            ip,
            video_port,
            audio_port,
            path_to_str(get_asset_path(test_sample).unwrap()).as_str(),
        ),
        TestSample::ElephantsDream => gst_stream(
            ip,
            video_port,
            audio_port,
            path_to_str(get_asset_path(test_sample).unwrap()).as_str(),
        ),
        TestSample::Sample => gst_stream(
            ip,
            video_port,
            audio_port,
            path_to_str(get_asset_path(test_sample).unwrap()).as_str(),
        ),
        TestSample::SampleLoop => Err(anyhow!(
            "cannot play sample in loop using gstreamer, try ffmpeg"
        )),
        TestSample::Generic => gst_stream_testsrc(ip, video_port, audio_port),
    }
}

fn path_to_str(path_buf: PathBuf) -> String {
    path_buf.into_os_string().into_string().unwrap()
}

fn get_asset_path(asset: TestSample) -> Option<PathBuf> {
    match asset {
        TestSample::BigBuckBunny => Some(get_path(
            "https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4",
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/examples/assets/BigBuckBunny.mp4"
            ),
        )),
        TestSample::ElephantsDream => Some(get_path(
            "http://commondatastorage.googleapis.com/gtv-videos-bucket/sample/ElephantsDream.mp4",
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/examples/assets/ElephantsDream.mp4"
            ),
        )),
        TestSample::Sample | TestSample::SampleLoop => Some(get_path(
            "https://filesamples.com/samples/video/mp4/sample_1280x720.mp4",
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/examples/assets/sample_1280_720.mp4"
            ),
        )),
        TestSample::Generic => None,
    }
}

fn get_path(file_url: &str, file_path: &str) -> PathBuf {
    if Path::new(file_path).exists() {
        return PathBuf::from(file_path);
    } else {
        return download_file(file_url, file_path).expect(&format!(
            "error while downloading asset from `{}`",
            file_url
        ));
    }
}
