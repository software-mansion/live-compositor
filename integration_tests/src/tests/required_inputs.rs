use std::{thread, time::Duration};

use crate::{
    compare_video_dumps, input_dump_from_disk, split_rtp_packet_dump, CommunicationProtocol,
    CompositorInstance, OutputReceiver, PacketSender,
};
use anyhow::Result;
use serde_json::json;

/// Required inputs with some packets delayed
///
/// Show `input_1` and `input_2` side by side for 20 seconds.
#[test]
pub fn required_inputs() -> Result<()> {
    const OUTPUT_DUMP_FILE: &str = "required_inputs_output.rtp";
    let instance = CompositorInstance::start();
    let input_1_port = instance.get_port();
    let input_2_port = instance.get_port();
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

    instance.send_request(json!({
        "type": "unregister",
        "entity_type": "output_stream",
        "output_id": "output_1",
        "schedule_time_ms": 20000,
    }))?;

    let output_receiver = OutputReceiver::start(output_port, CommunicationProtocol::Tcp)?;

    instance.send_request(json!({
        "type": "register",
        "entity_type": "rtp_input_stream",
        "transport_protocol": "tcp_server",
        "input_id": "input_1",
        "port": input_1_port,
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
        "port": input_2_port,
        "video": {
            "codec": "h264"
        },
        "required": true
    }))?;

    instance.send_request(json!({
        "type": "start",
    }))?;

    let mut input_1_sender = PacketSender::new(CommunicationProtocol::Tcp, input_1_port)?;
    let mut input_2_sender = PacketSender::new(CommunicationProtocol::Tcp, input_2_port)?;
    let input_1_dump = input_dump_from_disk("8_colors_input_video.rtp")?;
    let input_2_dump = input_dump_from_disk("8_colors_input_reversed_video.rtp")?;
    let (input_2_first_part, input_2_second_part) =
        split_rtp_packet_dump(input_2_dump, Duration::from_secs(1))?;

    input_1_sender.send(&input_1_dump)?;

    input_2_sender.send(&input_2_first_part)?;
    // Simulate delay in sending input_2 packets.
    thread::sleep(Duration::from_secs(2));
    input_2_sender.send(&input_2_second_part)?;

    let new_output_dump = output_receiver.wait_for_output()?;

    compare_video_dumps(
        OUTPUT_DUMP_FILE,
        &new_output_dump,
        &[Duration::from_millis(1200)],
        20.0,
    )?;

    Ok(())
}
