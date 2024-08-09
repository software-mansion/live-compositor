use std::{thread, time::Duration};

use crate::{
    compare_video_dumps, input_dump_from_disk, split_rtp_packet_dump, CommunicationProtocol,
    CompositorInstance, CompositorInstanceMode, OutputReceiver, PacketSender,
    VideoValidationConfig,
};
use anyhow::Result;
use serde_json::json;

/// Required inputs with some packets delayed.
/// No offset (it might required adding _flaky suffix)
///
/// Show `input_1` and `input_2` side by side for 20 seconds.
#[test]
pub fn required_inputs_no_offset() -> Result<()> {
    const OUTPUT_DUMP_FILE: &str = "required_inputs_no_offset_output.rtp";
    let instance = CompositorInstance::start(CompositorInstanceMode::RealTime);
    let input_1_port = instance.get_port();
    let input_2_port = instance.get_port();
    let output_port = instance.get_port();

    instance.send_request(
        "output/output_1/register",
        json!({
            "type": "rtp_stream",
            "transport_protocol": "tcp_server",
            "port": output_port,
            "video": {
                "resolution": {
                    "width": 640,
                    "height": 360,
                },
                "encoder": {
                    "type": "ffmpeg_h264",
                    "preset": "ultrafast",
                },
                "initial": {
                    "root": {
                        "type": "tiles",
                        "padding": 3,
                        "background_color_rgba": "#DDDDDDFF",
                        "children": [
                            {
                                "type": "input_stream",
                                "input_id": "input_1",
                            },
                            {
                                "type": "input_stream",
                                "input_id": "input_2",
                            },
                        ],
                    }
                }
            },
        }),
    )?;

    instance.send_request(
        "output/output_1/unregister",
        json!({
            "schedule_time_ms": 20000,
        }),
    )?;

    let output_receiver = OutputReceiver::start(output_port, CommunicationProtocol::Tcp)?;

    instance.send_request(
        "input/input_1/register",
        json!({
            "type": "rtp_stream",
            "transport_protocol": "tcp_server",
            "port": input_1_port,
            "video": {
                "decoder": "ffmpeg_h264"
            },
            "required": true,
        }),
    )?;

    instance.send_request(
        "input/input_2/register",
        json!({
            "type": "rtp_stream",
            "transport_protocol": "tcp_server",
            "port": input_2_port,
            "video": {
                "decoder": "ffmpeg_h264"
            },
            "required": true,
        }),
    )?;

    let input_1_sender = PacketSender::new(CommunicationProtocol::Tcp, input_1_port)?;
    let mut input_2_sender = PacketSender::new(CommunicationProtocol::Tcp, input_2_port)?;
    let input_1_dump = input_dump_from_disk("8_colors_input_video.rtp")?;
    let input_2_dump = input_dump_from_disk("8_colors_input_reversed_video.rtp")?;
    let (input_2_first_part, input_2_second_part) =
        split_rtp_packet_dump(input_2_dump, Duration::from_secs(1))?;

    let input_1_handle = input_1_sender.send_non_blocking(input_1_dump);

    let input_2_handle = thread::spawn(move || {
        input_2_sender.send(&input_2_first_part).unwrap();
        // Simulate delay in sending input_2 packets.
        thread::sleep(Duration::from_secs(3));
        input_2_sender.send(&input_2_second_part).unwrap();
    });

    instance.send_request("start", json!({}))?;

    input_1_handle.join().unwrap();
    input_2_handle.join().unwrap();
    let new_output_dump = output_receiver.wait_for_output()?;

    compare_video_dumps(
        OUTPUT_DUMP_FILE,
        &new_output_dump,
        VideoValidationConfig {
            validation_intervals: vec![Duration::ZERO..Duration::from_secs(18)],
            allowed_invalid_frames: 10,
            ..Default::default()
        },
    )?;

    Ok(())
}

/// Required inputs with some packets delayed. Offset set to 0.
///
/// Show `input_1` and `input_2` side by side for 20 seconds.
#[test]
pub fn required_inputs_with_offset() -> Result<()> {
    const OUTPUT_DUMP_FILE: &str = "required_inputs_with_offset_output.rtp";
    let instance = CompositorInstance::start(CompositorInstanceMode::RealTime);
    let input_1_port = instance.get_port();
    let input_2_port = instance.get_port();
    let output_port = instance.get_port();

    instance.send_request(
        "output/output_1/register",
        json!({
            "type": "rtp_stream",
            "transport_protocol": "tcp_server",
            "port": output_port,
            "video": {
                "resolution": {
                    "width": 640,
                    "height": 360,
                },
                "encoder": {
                    "type": "ffmpeg_h264",
                    "preset": "ultrafast",
                },
                "initial": {
                    "root": {
                        "type": "tiles",
                        "padding": 3,
                        "background_color_rgba": "#DDDDDDFF",
                        "children": [
                            {
                                "type": "input_stream",
                                "input_id": "input_1",
                            },
                            {
                                "type": "input_stream",
                                "input_id": "input_2",
                            },
                        ],
                    }
                }
            },
        }),
    )?;

    instance.send_request(
        "output/output_1/unregister",
        json!({
            "schedule_time_ms": 20000,
        }),
    )?;

    let output_receiver = OutputReceiver::start(output_port, CommunicationProtocol::Tcp)?;

    instance.send_request(
        "input/input_1/register",
        json!({
            "type": "rtp_stream",
            "transport_protocol": "tcp_server",
            "port": input_1_port,
            "video": {
                "decoder": "ffmpeg_h264"
            },
            "required": true,
            "offset_ms": 0,
        }),
    )?;

    instance.send_request(
        "input/input_2/register",
        json!({
            "type": "rtp_stream",
            "transport_protocol": "tcp_server",
            "port": input_2_port,
            "video": {
                "decoder": "ffmpeg_h264"
            },
            "required": true,
            "offset_ms": 0,
        }),
    )?;

    let input_1_sender = PacketSender::new(CommunicationProtocol::Tcp, input_1_port)?;
    let mut input_2_sender = PacketSender::new(CommunicationProtocol::Tcp, input_2_port)?;
    let input_1_dump = input_dump_from_disk("8_colors_input_video.rtp")?;
    let input_2_dump = input_dump_from_disk("8_colors_input_reversed_video.rtp")?;
    let (input_2_first_part, input_2_second_part) =
        split_rtp_packet_dump(input_2_dump, Duration::from_secs(1))?;

    let input_1_handle = input_1_sender.send_non_blocking(input_1_dump);

    let input_2_handle = thread::spawn(move || {
        input_2_sender.send(&input_2_first_part).unwrap();
        // Simulate delay in sending input_2 packets.
        thread::sleep(Duration::from_secs(3));
        input_2_sender.send(&input_2_second_part).unwrap();
    });

    instance.send_request("start", json!({}))?;

    input_1_handle.join().unwrap();
    input_2_handle.join().unwrap();
    let new_output_dump = output_receiver.wait_for_output()?;

    compare_video_dumps(
        OUTPUT_DUMP_FILE,
        &new_output_dump,
        VideoValidationConfig {
            validation_intervals: vec![Duration::ZERO..Duration::from_secs(18)],
            allowed_invalid_frames: 1,
            ..Default::default()
        },
    )?;

    Ok(())
}

/// Optional inputs with some packets delayed in the middle
/// No offset
///
/// Show `input_1` and `input_2` side by side for 1 second. `input_2` disappears for 3 seconds,
/// and then returns back to showing both stream.
#[test]
pub fn optional_inputs_no_offset_flaky() -> Result<()> {
    const OUTPUT_DUMP_FILE: &str = "optional_inputs_no_offset_output.rtp";
    let instance = CompositorInstance::start(CompositorInstanceMode::RealTime);
    let input_1_port = instance.get_port();
    let input_2_port = instance.get_port();
    let output_port = instance.get_port();

    instance.send_request(
        "output/output_1/register",
        json!({
            "type": "rtp_stream",
            "transport_protocol": "tcp_server",
            "port": output_port,
            "video": {
                "resolution": {
                    "width": 640,
                    "height": 360,
                },
                "encoder": {
                    "type": "ffmpeg_h264",
                    "preset": "ultrafast",
                },
                "initial": {
                    "root": {
                        "type": "tiles",
                        "padding": 3,
                        "background_color_rgba": "#DDDDDDFF",
                        "children": [
                            {
                                "type": "input_stream",
                                "input_id": "input_1",
                            },
                            {
                                "type": "input_stream",
                                "input_id": "input_2",
                            },
                        ],
                    }
                }
            },
        }),
    )?;

    instance.send_request(
        "output/output_1/unregister",
        json!({
            "schedule_time_ms": 20000,
        }),
    )?;

    let output_receiver = OutputReceiver::start(output_port, CommunicationProtocol::Tcp)?;

    instance.send_request(
        "input/input_1/register",
        json!({
            "type": "rtp_stream",
            "transport_protocol": "tcp_server",
            "port": input_1_port,
            "video": {
                "decoder": "ffmpeg_h264"
            },
        }),
    )?;

    instance.send_request(
        "input/input_2/register",
        json!({
            "type": "rtp_stream",
            "transport_protocol": "tcp_server",
            "port": input_2_port,
            "video": {
                "decoder": "ffmpeg_h264"
            },
        }),
    )?;

    let input_1_sender = PacketSender::new(CommunicationProtocol::Tcp, input_1_port)?;
    let mut input_2_sender = PacketSender::new(CommunicationProtocol::Tcp, input_2_port)?;
    let input_1_dump = input_dump_from_disk("8_colors_input_video.rtp")?;
    let input_2_dump = input_dump_from_disk("8_colors_input_reversed_video.rtp")?;
    let (input_2_first_part, input_2_second_part) =
        split_rtp_packet_dump(input_2_dump, Duration::from_secs(1))?;

    let input_1_handle = input_1_sender.send_non_blocking(input_1_dump);

    let input_2_handle = thread::spawn(move || {
        input_2_sender.send(&input_2_first_part).unwrap();
        // Simulate delay in sending input_2 packets.
        thread::sleep(Duration::from_secs(3));
        input_2_sender.send(&input_2_second_part).unwrap();
    });

    instance.send_request("start", json!({}))?;

    input_1_handle.join().unwrap();
    input_2_handle.join().unwrap();
    let new_output_dump = output_receiver.wait_for_output()?;

    compare_video_dumps(
        OUTPUT_DUMP_FILE,
        &new_output_dump,
        VideoValidationConfig {
            validation_intervals: vec![Duration::ZERO..Duration::from_secs(18)],
            allowed_invalid_frames: 10,
            ..Default::default()
        },
    )?;

    Ok(())
}
