use std::{thread, time::Duration};

use crate::{
    compare_video_dumps, input_dump_from_disk, output_dump_from_disk, split_rtp_packet_dump,
    CommunicationProtocol, CompositorInstance, OutputReceiver, PacketSender,
};
use anyhow::Result;
use serde_json::json;

#[test]
pub fn required_inputs() -> Result<()> {
    let instance = CompositorInstance::start(8010);

    instance.send_request(json!({
        "type": "register",
        "entity_type": "output_stream",
        "output_id": "output_1",
        "transport_protocol": "tcp_server",
        "port": 8011,
        "video": {
            "resolution": {
                "width": 1280,
                "height": 720,
            },
            "encoder_preset": "medium",
            "initial": {
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
        },
    }))?;

    let output_receiver = OutputReceiver::start(
        8011,
        CommunicationProtocol::Tcp,
        Duration::from_secs(10),
        "required_inputs_output.rtp",
    )?;

    instance.send_request(json!({
        "type": "register",
        "entity_type": "rtp_input_stream",
        "transport_protocol": "tcp_server",
        "input_id": "input_1",
        "port": 8012,
        "video": {
            "codec": "h264"
        },
        "required": true
    }))?;

    instance.send_request(json!({
        "type": "register",
        "entity_type": "rtp_input_stream",
        "transport_protocol": "tcp_server",
        "input_id": "input_2",
        "port": 8013,
        "video": {
            "codec": "h264"
        },
        "required": true
    }))?;

    instance.send_request(json!({
        "type": "start",
    }))?;

    let mut input_1_sender = PacketSender::new(CommunicationProtocol::Tcp, 8012)?;
    let mut input_2_sender = PacketSender::new(CommunicationProtocol::Tcp, 8013)?;
    let input_1_dump = input_dump_from_disk("8_colors_input_video.rtp")?;
    let input_2_dump = input_dump_from_disk("8_colors_input_reversed_video.rtp")?;
    let (input_2_first_part, input_2_second_part) =
        split_rtp_packet_dump(input_2_dump, Duration::from_secs(1))?;

    input_1_sender.send(&input_1_dump)?;

    input_2_sender.send(&input_2_first_part)?;
    thread::sleep(Duration::from_secs(2));
    input_2_sender.send(&input_2_second_part)?;

    // NOTE(noituri): Consider merging output_receiver.wait_for_output() and output_dump_from_disk() into a single function.
    let new_output_dump = output_receiver.wait_for_output()?;
    let output_dump_from_disk = output_dump_from_disk("required_inputs_output.rtp")?;

    compare_video_dumps(
        &output_dump_from_disk,
        &new_output_dump,
        &[Duration::from_millis(1200)],
        20.0,
    )?;

    Ok(())
}
