use anyhow::{anyhow, Result};

use std::{
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::Duration,
};

use super::utils::{get_asset_path, TestSample};

pub fn start_gst_receive_tcp(ip: &str, port: u16, video: bool, audio: bool) -> Result<()> {
    if !video && !audio {
        return Err(anyhow!(
            "At least one of: 'video_port', 'audio_port' has to be specified."
        ));
    }

    let mut gst_output_command = [
        "gst-launch-1.0 -v ",
        "rtpptdemux name=demux ",
        &format!("tcpclientsrc host={} port={} ! \"application/x-rtp-stream\" ! rtpstreamdepay ! queue ! demux. ", ip, port)
        ].concat();

    if !video && !audio {
        return Err(anyhow!(
            "At least one of: 'video_port', 'audio_port' has to be specified."
        ));
    }

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

pub fn start_gst_receive_udp(port: u16, video: bool, audio: bool) -> Result<()> {
    if !video && !audio {
        return Err(anyhow!(
            "At least one of: 'video_port', 'audio_port' has to be specified."
        ));
    }

    let mut gst_output_command = [
        "gst-launch-1.0 -v ",
        "rtpptdemux name=demux ",
        &format!(
            "udpsrc port={} ! \"application/x-rtp\" ! queue ! demux. ",
            port
        ),
    ]
    .concat();

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

pub fn start_gst_send_sample_tcp(
    ip: &str,
    video_port: Option<u16>,
    audio_port: Option<u16>,
    test_sample: TestSample,
) -> Result<()> {
    match test_sample {
        TestSample::BigBuckBunny | TestSample::ElephantsDream | TestSample::Sample => {
            start_gst_send_tcp(ip, video_port, audio_port, get_asset_path(test_sample)?)
        }
        TestSample::SampleLoop => Err(anyhow!(
            "cannot play sample in loop using gstreamer, try ffmpeg"
        )),
        TestSample::Generic => start_gst_send_testsrc_tcp(ip, video_port, audio_port),
    }
}

pub fn start_gst_send_sample_udp(
    ip: &str,
    video_port: Option<u16>,
    audio_port: Option<u16>,
    test_sample: TestSample,
) -> Result<()> {
    match test_sample {
        TestSample::BigBuckBunny | TestSample::ElephantsDream | TestSample::Sample => {
            start_gst_send_udp(ip, video_port, audio_port, get_asset_path(test_sample)?)
        }
        TestSample::SampleLoop => Err(anyhow!(
            "cannot play sample in loop using gstreamer, try ffmpeg"
        )),
        TestSample::Generic => start_gst_send_testsrc_udp(ip, video_port, audio_port),
    }
}

pub fn start_gst_send_tcp(
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

    let path = path.to_string_lossy();

    let mut gst_input_command = [
        "gst-launch-1.0 -v ",
        &format!("filesrc location={path} ! qtdemux name=demux "),
    ]
    .concat();

    if let Some(port) = video_port {
        gst_input_command = gst_input_command + &format!("demux.video_0 ! queue ! h264parse ! rtph264pay config-interval=1 !  application/x-rtp,payload=96  ! rtpstreampay ! tcpclientsink host={ip} port={port} ");
    }
    if let Some(port) = audio_port {
        gst_input_command = gst_input_command + &format!("demux.audio_0 ! queue ! decodebin ! audioconvert ! audioresample ! opusenc ! rtpopuspay ! application/x-rtp,payload=97 !  rtpstreampay ! tcpclientsink host={ip} port={port} ");
    }

    Command::new("bash")
        .arg("-c")
        .arg(gst_input_command)
        .spawn()?;

    Ok(())
}

pub fn start_gst_send_udp(
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

    let path = path.to_string_lossy();

    let mut gst_input_command = [
        "gst-launch-1.0 -v ",
        &format!("filesrc location={path} ! qtdemux name=demux "),
    ]
    .concat();

    if let Some(port) = video_port {
        gst_input_command = gst_input_command + &format!("demux.video_0 ! queue ! h264parse ! rtph264pay config-interval=1 !  application/x-rtp,payload=96  ! udpsink host={ip} port={port} ");
    }
    if let Some(port) = audio_port {
        gst_input_command = gst_input_command + &format!("demux.audio_0 ! queue ! decodebin ! audioconvert ! audioresample ! opusenc ! rtpopuspay ! application/x-rtp,payload=97 ! udpsink host={ip} port={port} ");
    }

    Command::new("bash")
        .arg("-c")
        .arg(gst_input_command)
        .spawn()?;

    Ok(())
}

pub fn start_gst_send_testsrc_tcp(
    ip: &str,
    video_port: Option<u16>,
    audio_port: Option<u16>,
) -> Result<()> {
    if video_port.is_none() && audio_port.is_none() {
        return Err(anyhow!(
            "At least one of: 'video_port', 'audio_port' has to be specified."
        ));
    }

    let mut gst_input_command = [
        "gst-launch-1.0 -v videotestsrc ! ",
        "\"video/x-raw,format=I420,width=1920,height=1080,framerate=60/1\" ! ",
    ]
    .concat();

    if let Some(port) = video_port {
        gst_input_command = gst_input_command + &format!(" x264enc tune=zerolatency speed-preset=superfast ! rtph264pay ! application/x-rtp,payload=96 ! rtpstreampay ! tcpclientsink host={ip} port={port}");
    }
    if let Some(port) = audio_port {
        gst_input_command = gst_input_command + &format!(" audiotestsrc ! audioconvert ! audioresample ! opusenc ! rtpopuspay ! application/x-rtp,payload=97 ! rtpstreampay ! tcpclientsink host={ip} port={port}");
    }

    Command::new("bash")
        .arg("-c")
        .arg(gst_input_command)
        .spawn()?;

    Ok(())
}

pub fn start_gst_send_testsrc_udp(
    ip: &str,
    video_port: Option<u16>,
    audio_port: Option<u16>,
) -> Result<()> {
    if video_port.is_none() && audio_port.is_none() {
        return Err(anyhow!(
            "At least one of: 'video_port', 'audio_port' has to be specified."
        ));
    }

    let mut gst_input_command = [
        "gst-launch-1.0 -v videotestsrc ! ",
        "\"video/x-raw,format=I420,width=1920,height=1080,framerate=60/1\" ! ",
    ]
    .concat();

    if let Some(port) = video_port {
        gst_input_command = gst_input_command + &format!(" x264enc tune=zerolatency speed-preset=superfast ! rtph264pay ! application/x-rtp,payload=96 ! udpsink host={ip} port={port}");
    }
    if let Some(port) = audio_port {
        gst_input_command = gst_input_command + &format!(" audiotestsrc ! audioconvert ! audioresample ! opusenc ! rtpopuspay ! application/x-rtp,payload=97 ! udpsink host={ip} port={port}");
    }

    Command::new("bash")
        .arg("-c")
        .arg(gst_input_command)
        .spawn()?;

    Ok(())
}
