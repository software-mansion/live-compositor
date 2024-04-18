use anyhow::Result;
use serde_json::json;
use std::time::Duration;

use crate::{
    compare_audio_dumps, input_dump_from_disk, AudioValidationConfig, CommunicationProtocol,
    CompositorInstance, OutputReceiver, PacketSender,
};

/// An AAC audio input stream.
///
/// Play audio for 10 seconds.
#[test]
pub fn aac() -> Result<()> {
    const OUTPUT_DUMP_FILE: &str = "aac_output.rtp";
    let instance = CompositorInstance::start();
    let input_1_port = instance.get_port();
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
                            "volume": 1.0,
                        },
                    ]
                },
                "encoder": {
                    "type": "opus",
                    "channels": "stereo",
                }
            },
        }),
    )?;

    instance.send_request(
        "output/output_1/unregister",
        json!({
            "schedule_time_ms": 10000,
        }),
    )?;

    instance.send_request(
        "input/input_1/register",
        json!({
            "type": "rtp_stream",
            "transport_protocol": "udp",
            "port": input_1_port,
            "audio": {
                "decoder": "aac",
                "audio_specific_config": "1210",
                "rtp_mode": "high_bitrate",
            }
        }),
    )?;

    let audio_input_1 = input_dump_from_disk("big_buck_bunny_10s_audio_aac.rtp")?;
    let mut audio_1_sender = PacketSender::new(CommunicationProtocol::Udp, input_1_port)?;

    audio_1_sender.send(&audio_input_1)?;

    instance.send_request("start", json!({}))?;

    let new_output_dump = output_receiver.wait_for_output()?;

    compare_audio_dumps(
        OUTPUT_DUMP_FILE,
        &new_output_dump,
        AudioValidationConfig {
            sampling_intervals: vec![
                Duration::from_millis(500)..Duration::from_millis(1500),
                Duration::from_millis(2500)..Duration::from_millis(3500),
            ],
            ..Default::default()
        },
    )?;

    Ok(())
}
