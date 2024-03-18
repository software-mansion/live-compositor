use crate::{
    compare_dumps, input_dump_from_disk, output_dump_from_disk, CommunicationProtocol,
    CompositorInstance, OutputReceiver, PacketSender,
};
use anyhow::Result;
use serde_json::json;
use std::time::Duration;

#[test]
fn simple() {
    crate::integration_test_prerequisites();
    run_simple_test(false).unwrap();
}

pub fn run_simple_test(update_dumps: bool) -> Result<()> {
    let mut instance = CompositorInstance::start(8000);

    instance.send_request(json!({
        "type": "register",
        "entity_type": "rtp_input_stream",
        "transport_protocol": "tcp_server",
        "input_id": "input_1",
        "port": 8004,
        "video": {
            "codec": "h264"
        }
    }))?;

    let output_receiver = OutputReceiver::start(
        8002,
        CommunicationProtocol::Udp,
        Duration::from_secs(4),
        "simple_scene_output.rtp",
        update_dumps,
    )?;

    instance.send_request(json!({
        "type": "register",
        "entity_type": "output_stream",
        "output_id": "output_1",
        "transport_protocol": "udp",
        "ip": "127.0.0.1",
        "port": 8002,
        "video": {
            "resolution": {
                "width": 1920,
                "height": 1080,
            },
            "encoder_preset": "medium",
            "initial": {
                "id": "input_1",
                "type": "input_stream",
                "input_id": "input_1",
            }
        }
    }))?;

    let packets_dump = input_dump_from_disk("8_colors_input.rtp")?;
    let mut sender = PacketSender::new(CommunicationProtocol::Tcp, 8004)?;
    sender.send(&packets_dump)?;

    instance.send_request(json!({
        "type": "start",
    }))?;

    let output_dump_from_disk = output_dump_from_disk("simple_scene_output.rtp")?;
    let new_output_dump = output_receiver.recv()?;

    compare_dumps(
        &output_dump_from_disk,
        &new_output_dump,
        (Duration::from_secs(0), Duration::from_secs(2)),
    )?;

    Ok(())
}
