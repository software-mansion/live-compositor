use anyhow::{anyhow, Result};
use live_compositor::types::Resolution;

use super::examples::{get_asset_path, TestSample};
use std::{
    fs::File,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::Duration,
};

pub fn start_ffmpeg_receive(video_port: Option<u16>, audio_port: Option<u16>) -> Result<()> {
    let output_sdp_path = match (video_port, audio_port) {
        (Some(video_port), Some(audio_port)) => {
            write_video_audio_example_sdp_file(video_port, audio_port)
        }
        (Some(video_port), None) => write_video_example_sdp_file(video_port),
        (None, Some(audio_port)) => write_audio_example_sdp_file(audio_port),
        (None, None) => {
            return Err(anyhow!(
                "At least one of: 'video_port', 'audio_port' has to be specified."
            ))
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

pub fn start_ffmpeg_send_sample(
    ip: &str,
    video_port: Option<u16>,
    audio_port: Option<u16>,
    test_sample: TestSample,
) -> Result<()> {
    match test_sample {
        TestSample::BigBuckBunny | TestSample::ElephantsDream => {
            start_ffmpeg_send(ip, video_port, audio_port, get_asset_path(test_sample)?)
        }
        TestSample::Sample => match video_port {
            Some(port) => start_ffmpeg_send_video(ip, port, get_asset_path(test_sample)?),
            None => Err(anyhow!("video port required for test sample")),
        },
        TestSample::SampleLoop => match video_port {
            Some(port) => start_ffmpeg_send_video_loop(ip, port, get_asset_path(test_sample)?),
            None => Err(anyhow!("video port required for test sample")),
        },
        TestSample::Generic => match video_port {
            Some(port) => start_ffmpeg_send_testsrc(
                ip,
                port,
                Resolution {
                    width: 1920,
                    height: 1080,
                },
            ),
            None => Err(anyhow!("video port required for generic")),
        },
    }
}

pub fn start_ffmpeg_send(
    ip: &str,
    video_port: Option<u16>,
    audio_port: Option<u16>,
    path: PathBuf,
) -> Result<()> {
    if video_port.is_none() && audio_port.is_none() {
        return Err(anyhow!(
            "At least one of: 'video_port', 'audio_port' has to be specified."
        ));
    }

    if let Some(port) = video_port {
        start_ffmpeg_send_video(ip, port, path.clone())?;
    }
    if let Some(port) = audio_port {
        start_ffmpeg_send_audio(ip, port, path, "libopus")?
    }

    Ok(())
}

pub fn start_ffmpeg_send_video(ip: &str, port: u16, path: PathBuf) -> Result<()> {
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
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    Ok(())
}

pub fn start_ffmpeg_send_video_loop(ip: &str, port: u16, path: PathBuf) -> Result<()> {
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
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    Ok(())
}

pub fn start_ffmpeg_send_audio(ip: &str, port: u16, path: PathBuf, codec: &str) -> Result<()> {
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
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    Ok(())
}

pub fn start_ffmpeg_send_testsrc(ip: &str, port: u16, resolution: Resolution) -> Result<()> {
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
fn write_video_example_sdp_file(port: u16) -> Result<String> {
    let ip = "127.0.0.1";
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
fn write_video_audio_example_sdp_file(video_port: u16, audio_port: u16) -> Result<String> {
    let ip = "127.0.0.1";
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

/// The SDP file will describe an RTP session on localhost with Opus audio encoding.
fn write_audio_example_sdp_file(port: u16) -> Result<String> {
    let ip = "127.0.0.1";
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
