use anyhow::Result;
use serde_json::json;
use std::time::Duration;

use crate::{
    audio_decoder::AudioChannels, compare_audio_dumps, input_dump_from_disk, output_dump_from_disk,
    CommunicationProtocol, CompositorInstance, OutputReceiver, PacketSender,
};

#[test]
pub fn audio_mixing() -> Result<()> {
    let instance = CompositorInstance::start(8030);

    let output_receiver = OutputReceiver::start(
        8031,
        CommunicationProtocol::Udp,
        Duration::from_secs(10),
        "audio_mixing_output.rtp",
    )?;

    instance.send_request(json!({
        "type": "register",
        "entity_type": "output_stream",
        "output_id": "output_1",
        "transport_protocol": "udp",
        "ip": "127.0.0.1",
        "port": 8031,
        "audio": {
            "initial": {
                "inputs": [
                    {
                        "input_id": "input_1",
                        "volume": 0.3,
                    },
                    {
                        "input_id": "input_2",
                        "volume": 0.7,
                    }
                ]
            },
            "channels": "stereo"
        }
    }))?;

    instance.send_request(json!({
        "type": "register",
        "entity_type": "rtp_input_stream",
        "transport_protocol": "tcp_server",
        "input_id": "input_1",
        "port": 8032,
        "audio": {
            "codec": "opus"
        }
    }))?;

    instance.send_request(json!({
        "type": "register",
        "entity_type": "rtp_input_stream",
        "transport_protocol": "udp",
        "input_id": "input_2",
        "port": 8033,
        "audio": {
            "codec": "opus"
        }
    }))?;

    let audio_input_1 = input_dump_from_disk("8_colors_input_audio.rtp")?;
    let audio_input_2 = input_dump_from_disk("8_colors_input_reversed_audio.rtp")?;
    let mut audio_1_sender = PacketSender::new(CommunicationProtocol::Tcp, 8032)?;
    let mut audio_2_sender = PacketSender::new(CommunicationProtocol::Udp, 8033)?;

    audio_1_sender.send(&audio_input_1)?;
    audio_2_sender.send(&audio_input_2)?;

    instance.send_request(json!({
        "type": "start",
    }))?;

    let new_output_dump = output_receiver.wait_for_output()?;
    let output_dump_from_disk = output_dump_from_disk("audio_mixing_output.rtp")?;

    compare_audio_dumps(
        &output_dump_from_disk,
        &new_output_dump,
        &[
            Duration::from_millis(500)..Duration::from_millis(1500),
            Duration::from_millis(2500)..Duration::from_millis(3500),
        ],
        4.0,
        AudioChannels::Stereo,
    )?;

    Ok(())
}
