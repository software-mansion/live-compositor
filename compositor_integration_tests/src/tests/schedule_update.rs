use std::time::Duration;

use crate::{
    compare_video_dumps, input_dump_from_disk, output_dump_from_disk, CommunicationProtocol,
    CompositorInstance, OutputReceiver, PacketSender,
};
use anyhow::Result;
use serde_json::json;

pub fn schedule_update() -> Result<()> {
    let instance = CompositorInstance::start(8040);

    instance.send_request(json!({
        "type": "register",
        "entity_type": "output_stream",
        "output_id": "output_1",
        "transport_protocol": "tcp_server",
        "port": 8041,
        "video": {
            "resolution": {
                "width": 1920,
                "height": 1080,
            },
            "encoder_preset": "medium",
            "initial": {
                "type": "tiles",
                "id": "tiles_1",
                "padding": 3,
                "background_color_rgba": "#DDDDDDFF",
                "transition": {
                    "duration_ms": 500,
                    "easing_function": {
                        "function_name": "bounce"
                    }
                },
                "children": [
                    {
                        "type": "input_stream",
                        "input_id": "input_1",
                    },
                ],
            }
        },
    }))?;

    instance.send_request(json!({
        "type": "update_output",
        "output_id": "output_1",
        "video": {
            "type": "tiles",
            "id": "tiles_1",
            "padding": 3,
            "background_color_rgba": "#DDDDDDFF",
            "transition": {
                "duration_ms": 500,
                "easing_function": {
                    "function_name": "bounce"
                }
            },
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
        },
        "schedule_time_ms": 2000
    }))?;

    let output_receiver = OutputReceiver::start(
        8041,
        CommunicationProtocol::Tcp,
        Duration::from_secs(5),
        "schedule_update_output.rtp",
    )?;

    instance.send_request(json!({
        "type": "register",
        "entity_type": "rtp_input_stream",
        "transport_protocol": "udp",
        "input_id": "input_1",
        "port": 8042,
        "video": {
            "codec": "h264"
        },
    }))?;

    instance.send_request(json!({
        "type": "register",
        "entity_type": "rtp_input_stream",
        "transport_protocol": "tcp_server",
        "input_id": "input_2",
        "port": 8043,
        "video": {
            "codec": "h264"
        },
    }))?;

    instance.send_request(json!({
        "type": "start",
    }))?;

    let mut input_1_sender = PacketSender::new(CommunicationProtocol::Udp, 8042)?;
    let mut input_2_sender = PacketSender::new(CommunicationProtocol::Tcp, 8043)?;
    let input_1_dump = input_dump_from_disk("8_colors_input_video.rtp")?;
    let input_2_dump = input_dump_from_disk("8_colors_input_reversed_video.rtp")?;

    input_1_sender.send(&input_1_dump)?;
    input_2_sender.send(&input_2_dump)?;

    let new_output_dump = output_receiver.wait_for_output()?;
    let output_dump_from_disk = output_dump_from_disk("schedule_update_output.rtp")?;

    compare_video_dumps(
        &output_dump_from_disk,
        &new_output_dump,
        &[Duration::from_millis(500), Duration::from_micros(3500)],
        20.0,
    )?;

    Ok(())
}
