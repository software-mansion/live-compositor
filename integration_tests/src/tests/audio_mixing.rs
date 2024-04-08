use anyhow::Result;
use serde_json::json;
use std::time::Duration;

use crate::{
    audio_decoder::AudioChannels, compare_audio_dumps, input_dump_from_disk, CommunicationProtocol,
    CompositorInstance, OutputReceiver, PacketSender,
};

/// Two audio input streams mixed together with different volumes.
///
/// Play mixed audio for 20 seconds.
#[test]
pub fn audio_mixing() -> Result<()> {
    const OUTPUT_DUMP_FILE: &str = "audio_mixing_output.rtp";
    let instance = CompositorInstance::start();
    let input_1_port = instance.get_port();
    let input_2_port = instance.get_port();
    let output_port = instance.get_port();

    let output_receiver = OutputReceiver::start(output_port, CommunicationProtocol::Udp)?;

    instance.send_request(
        "output/output_1/register",
        json!({
            "type": "rtp_stream",
            "transport_protocol": "udp",
            "ip": "127.0.0.1",
            "port": output_port,
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
                "channels": "stereo",
            },
        }),
    )?;

    instance.send_request(
        "output/output_1/unregister",
        json!({
            "schedule_time_ms": 20000,
        }),
    )?;

    instance.send_request(
        "input/input_1/register",
        json!({
            "type": "rtp_stream",
            "transport_protocol": "tcp_server",
            "port": input_1_port,
            "audio": {
                "codec": "opus"
            }
        }),
    )?;

    instance.send_request(
        "input/input_2/register",
        json!({
            "type": "rtp_stream",
            "transport_protocol": "udp",
            "port": input_2_port,
            "audio": {
                "codec": "opus"
            }
        }),
    )?;

    let audio_input_1 = input_dump_from_disk("countdown_audio.rtp")?;
    let audio_input_2 = input_dump_from_disk("8_colors_input_reversed_audio.rtp")?;
    let mut audio_1_sender = PacketSender::new(CommunicationProtocol::Tcp, input_1_port)?;
    let mut audio_2_sender = PacketSender::new(CommunicationProtocol::Udp, input_2_port)?;

    audio_1_sender.send(&audio_input_1)?;
    audio_2_sender.send(&audio_input_2)?;

    instance.send_request("start", json!({}))?;

    let new_output_dump = output_receiver.wait_for_output()?;

    compare_audio_dumps(
        OUTPUT_DUMP_FILE,
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
