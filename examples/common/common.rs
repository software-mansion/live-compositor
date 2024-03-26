use anyhow::{anyhow, Result};

use crossbeam_channel::unbounded;
use log::error;
use reqwest::{blocking::Response, StatusCode};
use std::{
    env,
    fs::{self, File},
    io::{self, Write},
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::Duration,
};
use tracing::info;
use video_compositor::{config::config, types::Resolution};
use websocket::{Message, OwnedMessage};

use serde::Serialize;

pub fn post<T: Serialize + ?Sized>(json: &T) -> Result<Response> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .post(format!("http://127.0.0.1:{}/--/api", config().api_port))
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

#[allow(dead_code)]
pub fn start_websocket_thread() {
    let client = websocket::sync::client::ClientBuilder::new(&format!(
        "ws://127.0.0.1:{}/--/ws",
        config().api_port
    ))
    .unwrap()
    .connect_insecure()
    .unwrap();

    let (mut receiver, mut sender) = client.split().unwrap();

    let (tx, rx) = unbounded();

    thread::spawn(move || {
        for message in rx {
            if let OwnedMessage::Close(_) = message {
                let _ = sender.send_message(&message);
                return;
            }
            match sender.send_message(&message) {
                Ok(()) => (),
                Err(e) => {
                    println!("Send Loop: {:?}", e);
                    let _ = sender.send_message(&Message::close());
                    return;
                }
            }
        }
    });

    thread::spawn(move || {
        for message in receiver.incoming_messages() {
            match message {
                Ok(OwnedMessage::Close(_)) => {
                    let _ = tx.send(OwnedMessage::Close(None));
                    return;
                }
                Ok(OwnedMessage::Ping(data)) => {
                    if tx.send(OwnedMessage::Pong(data)).is_err() {
                        return;
                    }
                }
                Err(_) => {
                    let _ = tx.send(OwnedMessage::Close(None));
                    return;
                }
                _ => info!("Received compositor event: {:?}", message),
            }
        }
    });
}

#[allow(dead_code)]
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

#[allow(dead_code)]
pub fn start_ffplay(ip: &str, video_port: u16, audio_port: Option<u16>) -> Result<()> {
    let output_sdp_path = match audio_port {
        Some(audio_port) => write_video_audio_example_sdp_file(ip, video_port, audio_port),
        None => write_video_example_sdp_file(ip, video_port),
    }?;

    Command::new("ffplay")
        .args(["-protocol_whitelist", "file,rtp,udp", &output_sdp_path])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    thread::sleep(Duration::from_secs(2));

    Ok(())
}

#[allow(dead_code)]
pub fn stream_video(ip: &str, port: u16, path: PathBuf) -> Result<()> {
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

#[allow(dead_code)]
pub fn stream_audio(ip: &str, port: u16, path: PathBuf) -> Result<()> {
    Command::new("ffmpeg")
        .args(["-stream_loop", "-1", "-re", "-i"])
        .arg(path.clone())
        .args([
            "-vn",
            "-c:a",
            "libopus",
            "-f",
            "rtp",
            &format!("rtp://{ip}:{port}?rtcpport={port}"),
        ])
        .spawn()?;

    Ok(())
}

#[allow(dead_code)]
pub fn stream_ffmpeg_testsrc(ip: &str, port: u16, resolution: Resolution) -> Result<()> {
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
