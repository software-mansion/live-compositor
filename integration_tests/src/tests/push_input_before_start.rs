use std::{thread, time::Duration};

use anyhow::Result;
use serde_json::json;

use crate::{
    compare_video_dumps, input_dump_from_disk, CommunicationProtocol, CompositorInstance,
    OutputReceiver, PacketSender,
};

/// Check if the input stream is passed to the output correctly even if entire
/// stream was delivered before the compositor start. (TCP input)
///
/// Output:
/// - Display entire input stream from the beginning (16 seconds). Not black frames at the
/// beginning. Starts with a green.
/// - Black screen for remaining 14 seconds.
#[test]
pub fn push_entire_input_before_start_tcp() -> Result<()> {
    const OUTPUT_DUMP_FILE: &str = "push_entire_input_before_start_tcp.rtp";
    let instance = CompositorInstance::start();
    let input_port = instance.get_port();
    let output_port = instance.get_port();

    instance.send_request(json!({
        "type": "register",
        "entity_type": "output_stream",
        "output_id": "output_1",
        "transport_protocol": "tcp_server",
        "port": output_port,
        "video": {
            "resolution": {
                "width": 640,
                "height": 360,
            },
            "encoder_preset": "ultrafast",
            "initial": {
                "type": "input_stream",
                "input_id": "input_1",
            }
        },
    }))?;

    let output_receiver = OutputReceiver::start(
        output_port,
        CommunicationProtocol::Tcp,
        Duration::from_secs(30),
    )?;

    instance.send_request(json!({
        "type": "register",
        "entity_type": "rtp_input_stream",
        "transport_protocol": "tcp_server",
        "input_id": "input_1",
        "port": input_port,
        "video": {
            "codec": "h264"
        },
        "offset_ms": 0
    }))?;

    let mut input_1_sender = PacketSender::new(CommunicationProtocol::Tcp, input_port)?;
    let input_1_dump = input_dump_from_disk("8_colors_input_video.rtp")?;

    input_1_sender.send(&input_1_dump)?;

    thread::sleep(Duration::from_secs(5));

    instance.send_request(json!({
        "type": "start",
    }))?;

    let new_output_dump = output_receiver.wait_for_output()?;

    compare_video_dumps(
        OUTPUT_DUMP_FILE,
        &new_output_dump,
        &[Duration::from_millis(1200)],
        20.0,
    )?;

    Ok(())
}

/// Check if the input stream is passed to the output correctly even if entire
/// stream was delivered before the compositor start. (UDP)
///
/// Output:
/// - Display entire input stream from the beginning (16 seconds). No black frames at the
/// beginning. Starts with a green screen.
/// - Black screen for remaining 14 seconds.
#[test]
pub fn push_entire_input_before_start_udp() -> Result<()> {
    const OUTPUT_DUMP_FILE: &str = "push_entire_input_before_start_udp.rtp";
    let instance = CompositorInstance::start();
    let input_port = instance.get_port();
    let output_port = instance.get_port();

    instance.send_request(json!({
        "type": "register",
        "entity_type": "output_stream",
        "output_id": "output_1",
        "transport_protocol": "tcp_server",
        "port": output_port,
        "video": {
            "resolution": {
                "width": 640,
                "height": 360,
            },
            "encoder_preset": "ultrafast",
            "initial": {
                "type": "input_stream",
                "input_id": "input_1",
            }
        },
    }))?;

    let output_receiver = OutputReceiver::start(
        output_port,
        CommunicationProtocol::Tcp,
        Duration::from_secs(30),
    )?;

    instance.send_request(json!({
        "type": "register",
        "entity_type": "rtp_input_stream",
        "transport_protocol": "udp",
        "input_id": "input_1",
        "port": input_port,
        "video": {
            "codec": "h264"
        },
        "offset_ms": 0
    }))?;

    let mut input_1_sender = PacketSender::new(CommunicationProtocol::Udp, input_port)?;
    let input_1_dump = input_dump_from_disk("8_colors_input_video.rtp")?;

    input_1_sender.send(&input_1_dump)?;

    thread::sleep(Duration::from_secs(5));

    instance.send_request(json!({
        "type": "start",
    }))?;

    let new_output_dump = output_receiver.wait_for_output()?;

    compare_video_dumps(
        OUTPUT_DUMP_FILE,
        &new_output_dump,
        &[Duration::from_millis(1200)],
        20.0,
    )?;

    Ok(())
}

/// Check if the input stream is processed correctly if the stream is delivered few seconds before
/// queue start. Test case where there is no offset defined. (TCP server)
///
/// Output:
/// - Display input stream without initial 5 seconds from the beginning (11 seconds). Not black frames at the
/// beginning. Starts with a red color.
/// - Black screen for remaining 19 seconds.
#[test]
pub fn push_entire_input_before_start_tcp_without_offset() -> Result<()> {
    const OUTPUT_DUMP_FILE: &str = "push_entire_input_before_start_tcp_without_offset.rtp";
    let instance = CompositorInstance::start();
    let input_port = instance.get_port();
    let output_port = instance.get_port();

    instance.send_request(json!({
        "type": "register",
        "entity_type": "output_stream",
        "output_id": "output_1",
        "transport_protocol": "tcp_server",
        "port": output_port,
        "video": {
            "resolution": {
                "width": 640,
                "height": 360,
            },
            "encoder_preset": "ultrafast",
            "initial": {
                "type": "input_stream",
                "input_id": "input_1",
            }
        },
    }))?;

    let output_receiver = OutputReceiver::start(
        output_port,
        CommunicationProtocol::Tcp,
        Duration::from_secs(30),
    )?;

    instance.send_request(json!({
        "type": "register",
        "entity_type": "rtp_input_stream",
        "transport_protocol": "tcp_server",
        "input_id": "input_1",
        "port": input_port,
        "video": {
            "codec": "h264"
        },
    }))?;

    let mut input_1_sender = PacketSender::new(CommunicationProtocol::Tcp, input_port)?;
    let input_1_dump = input_dump_from_disk("8_colors_input_video.rtp")?;

    input_1_sender.send(&input_1_dump)?;

    thread::sleep(Duration::from_secs(5));

    instance.send_request(json!({
        "type": "start",
    }))?;

    let new_output_dump = output_receiver.wait_for_output()?;

    compare_video_dumps(
        OUTPUT_DUMP_FILE,
        &new_output_dump,
        &[Duration::from_millis(1200)],
        20.0,
    )?;

    Ok(())
}

/// Check if the input stream is processed correctly if the stream is delivered few seconds before
/// queue start. Test case where there is no offset defined. (UPD)
///
/// Output:
/// - Display entire input stream from the beginning (16 seconds). No black frames at the
/// beginning. Starts with a red color.
/// - Black screen for remaining 14 seconds.
#[test]
pub fn push_entire_input_before_start_udp_without_offset() -> Result<()> {
    const OUTPUT_DUMP_FILE: &str = "push_entire_input_before_start_udp_without_offset.rtp";
    let instance = CompositorInstance::start();
    let input_port = instance.get_port();
    let output_port = instance.get_port();

    instance.send_request(json!({
        "type": "register",
        "entity_type": "output_stream",
        "output_id": "output_1",
        "transport_protocol": "tcp_server",
        "port": output_port,
        "video": {
            "resolution": {
                "width": 640,
                "height": 360,
            },
            "encoder_preset": "ultrafast",
            "initial": {
                "type": "input_stream",
                "input_id": "input_1",
            }
        },
    }))?;

    let output_receiver = OutputReceiver::start(
        output_port,
        CommunicationProtocol::Tcp,
        Duration::from_secs(30),
    )?;

    instance.send_request(json!({
        "type": "register",
        "entity_type": "rtp_input_stream",
        "transport_protocol": "udp",
        "input_id": "input_1",
        "port": input_port,
        "video": {
            "codec": "h264"
        },
    }))?;

    let mut input_1_sender = PacketSender::new(CommunicationProtocol::Udp, input_port)?;
    let input_1_dump = input_dump_from_disk("8_colors_input_video.rtp")?;

    input_1_sender.send(&input_1_dump)?;

    thread::sleep(Duration::from_secs(5));

    instance.send_request(json!({
        "type": "start",
    }))?;

    let new_output_dump = output_receiver.wait_for_output()?;

    compare_video_dumps(
        OUTPUT_DUMP_FILE,
        &new_output_dump,
        &[Duration::from_millis(1200)],
        20.0,
    )?;

    Ok(())
}
