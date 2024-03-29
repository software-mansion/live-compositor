use std::{thread, time::Duration};

use crate::{
    compare_video_dumps, input_dump_from_disk, output_dump_from_disk, CommunicationProtocol,
    CompositorInstance, OutputReceiver, PacketSender,
};
use anyhow::Result;
use serde_json::json;

/// Check if the input stream is passed to the output correctly even if entire 
/// stream was delivered before the compositor start. (TCP input)
pub fn push_entire_input_before_start_tcp() -> Result<()> {
    const OUTPUT_DUMP_FILE: &str = "push_entire_input_before_start_tcp.rtp";
    let instance = CompositorInstance::start(8050);

    instance.send_request(json!({
        "type": "register",
        "entity_type": "output_stream",
        "output_id": "output_1",
        "transport_protocol": "tcp_server",
        "port": 8051,
        "video": {
            "resolution": {
                "width": 1920,
                "height": 1080,
            },
            "encoder_preset": "medium",
            "initial": {
                "type": "input_stream",
                "input_id": "input_1",
            }
        },
    }))?;

    let output_receiver = OutputReceiver::start(
        8051,
        CommunicationProtocol::Tcp,
        Duration::from_secs(30),
        OUTPUT_DUMP_FILE,
    )?;

    instance.send_request(json!({
        "type": "register",
        "entity_type": "rtp_input_stream",
        "transport_protocol": "tcp_server",
        "input_id": "input_1",
        "port": 8052,
        "video": {
            "codec": "h264"
        },
        "offset_ms": 0
    }))?;

    let mut input_1_sender = PacketSender::new(CommunicationProtocol::Tcp, 8052)?;
    let input_1_dump = input_dump_from_disk("8_colors_input_video.rtp")?;

    input_1_sender.send(&input_1_dump)?;
    
    thread::sleep(Duration::from_secs(5));

    instance.send_request(json!({
        "type": "start",
    }))?;

    let new_output_dump = output_receiver.wait_for_output()?;
    let output_dump_from_disk = output_dump_from_disk(OUTPUT_DUMP_FILE)?;

    // Show input stream for about 15 second and fallback to black screen after that.
    compare_video_dumps(
        &output_dump_from_disk,
        &new_output_dump,
        &[Duration::from_millis(1200)],
        20.0,
    )?;

    Ok(())
}

/// Check if the input stream is passed to the output correctly even if entire 
/// stream was delivered before the compositor start. (UDP input)
pub fn push_entire_input_before_start_udp() -> Result<()> {
    const OUTPUT_DUMP_FILE: &str = "push_entire_input_before_start_udp.rtp";
    let instance = CompositorInstance::start(8060);

    instance.send_request(json!({
        "type": "register",
        "entity_type": "output_stream",
        "output_id": "output_1", "transport_protocol": "tcp_server",
        "port": 8061,
        "video": {
            "resolution": {
                "width": 1920,
                "height": 1080,
            },
            "encoder_preset": "medium",
            "initial": {
                "type": "input_stream",
                "input_id": "input_1",
            }
        },
    }))?;

    let output_receiver = OutputReceiver::start(
        8061,
        CommunicationProtocol::Tcp,
        Duration::from_secs(30),
        OUTPUT_DUMP_FILE,
    )?;

    instance.send_request(json!({
        "type": "register",
        "entity_type": "rtp_input_stream",
        "transport_protocol": "udp",
        "input_id": "input_1",
        "port": 8062,
        "video": {
            "codec": "h264"
        },
        "offset_ms": 0
    }))?;

    let mut input_1_sender = PacketSender::new(CommunicationProtocol::Udp, 8062)?;
    let input_1_dump = input_dump_from_disk("8_colors_input_video.rtp")?;

    input_1_sender.send(&input_1_dump)?;

    thread::sleep(Duration::from_secs(5));

    instance.send_request(json!({
        "type": "start",
    }))?;

    let new_output_dump = output_receiver.wait_for_output()?;
    let output_dump_from_disk = output_dump_from_disk(OUTPUT_DUMP_FILE)?;

    // Show entire input stream for about 15 second and fallback to black screen after that.
    compare_video_dumps(
        &output_dump_from_disk,
        &new_output_dump,
        &[Duration::from_millis(1200)],
        20.0,
    )?;

    Ok(())
}

/// Check if the input stream is passed to the output correctly even if entire 
/// stream was delivered before the compositor start. (TCP input without an offset)
pub fn push_entire_input_before_start_tcp_without_offset() -> Result<()> {
    const OUTPUT_DUMP_FILE: &str = "push_entire_input_before_start_tcp_without_offset.rtp";
    let instance = CompositorInstance::start(8070);

    instance.send_request(json!({
        "type": "register",
        "entity_type": "output_stream",
        "output_id": "output_1",
        "transport_protocol": "tcp_server",
        "port": 8071,
        "video": {
            "resolution": {
                "width": 1920,
                "height": 1080,
            },
            "encoder_preset": "medium",
            "initial": {
                "type": "input_stream",
                "input_id": "input_1",
            }
        },
    }))?;

    let output_receiver = OutputReceiver::start(
        8071,
        CommunicationProtocol::Tcp,
        Duration::from_secs(30),
        OUTPUT_DUMP_FILE,
    )?;

    instance.send_request(json!({
        "type": "register",
        "entity_type": "rtp_input_stream",
        "transport_protocol": "tcp_server",
        "input_id": "input_1",
        "port": 8072,
        "video": {
            "codec": "h264"
        },
    }))?;

    let mut input_1_sender = PacketSender::new(CommunicationProtocol::Tcp, 8072)?;
    let input_1_dump = input_dump_from_disk("8_colors_input_video.rtp")?;

    input_1_sender.send(&input_1_dump)?;
    thread::sleep(Duration::from_secs(5));

    instance.send_request(json!({
        "type": "start",
    }))?;

    let new_output_dump = output_receiver.wait_for_output()?;
    let output_dump_from_disk = output_dump_from_disk(OUTPUT_DUMP_FILE)?;

    // Show input stream for about 15 second and fallback to black screen after that.
    compare_video_dumps(
        &output_dump_from_disk,
        &new_output_dump,
        &[Duration::from_millis(1200)],
        20.0,
    )?;

    Ok(())
}

/// Check if the input stream is passed to the output correctly even if entire 
/// stream was delivered before the compositor start. (UDP input without an offset)
pub fn push_entire_input_before_start_udp_without_offset() -> Result<()> {
    const OUTPUT_DUMP_FILE: &str = "push_entire_input_before_start_udp_without_offset.rtp";
    let instance = CompositorInstance::start(8080);

    instance.send_request(json!({
        "type": "register",
        "entity_type": "output_stream",
        "output_id": "output_1", "transport_protocol": "tcp_server",
        "port": 8081,
        "video": {
            "resolution": {
                "width": 1920,
                "height": 1080,
            },
            "encoder_preset": "ultrafast",
            "initial": {
                "type": "input_stream",
                "input_id": "input_1",
            }
        },
    }))?;

    let output_receiver = OutputReceiver::start(
        8081,
        CommunicationProtocol::Tcp,
        Duration::from_secs(30),
        OUTPUT_DUMP_FILE,
    )?;

    instance.send_request(json!({
        "type": "register",
        "entity_type": "rtp_input_stream",
        "transport_protocol": "udp",
        "input_id": "input_1",
        "port": 8082,
        "video": {
            "codec": "h264"
        },
    }))?;

    let mut input_1_sender = PacketSender::new(CommunicationProtocol::Udp, 8082)?;
    let input_1_dump = input_dump_from_disk("8_colors_input_video.rtp")?;

    input_1_sender.send(&input_1_dump)?;

    thread::sleep(Duration::from_secs(5));

    instance.send_request(json!({
        "type": "start",
    }))?;

    let new_output_dump = output_receiver.wait_for_output()?;
    let output_dump_from_disk = output_dump_from_disk(OUTPUT_DUMP_FILE)?;

    // Should show most of the input stream (over 5 seconds should be missing from the beginning).
    // After that there will be only black frames until the end.
    compare_video_dumps(
        &output_dump_from_disk,
        &new_output_dump,
        &[Duration::from_millis(1200)],
        20.0,
    )?;

    Ok(())
}
