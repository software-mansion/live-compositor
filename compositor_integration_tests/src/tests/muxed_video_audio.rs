use anyhow::Result;
use serde_json::json;
use std::time::Duration;

use crate::{
    audio_decoder::AudioChannels, compare_audio_dumps, compare_video_dumps, input_dump_from_disk,
    output_dump_from_disk, CommunicationProtocol, CompositorInstance, OutputReceiver, PacketSender,
};

pub fn muxed_video_audio() -> Result<()> {
    let instance = CompositorInstance::start(8000);

    let output_receiver = OutputReceiver::start(
        8001,
        CommunicationProtocol::Udp,
        Duration::from_secs(10),
        "muxed_video_audio_output.rtp",
    )?;

    instance.send_request(json!({
        "type": "register",
        "entity_type": "output_stream",
        "output_id": "output_1",
        "transport_protocol": "udp",
        "ip": "127.0.0.1",
        "port": 8001,
        "video": {
            "resolution": {
                "width": 1280,
                "height": 720,
            },
            "encoder_preset": "medium",
            "initial": {
                "id": "input_1",
                "type": "input_stream",
                "input_id": "input_1",
            }
        },
        "audio": {
            "initial": {
                "inputs": [
                    {
                        "input_id": "input_1",
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
        "port": 8002,
        "video": {
            "codec": "h264"
        },
        "audio": {
            "codec": "opus"
        }
    }))?;

    let packets_dump = input_dump_from_disk("8_colors_input_video_audio.rtp")?;
    let mut packet_sender = PacketSender::new(CommunicationProtocol::Tcp, 8002)?;
    packet_sender.send(&packets_dump)?;

    instance.send_request(json!({
        "type": "start",
    }))?;

    let new_output_dump = output_receiver.wait_for_output()?;
    let output_dump_from_disk = output_dump_from_disk("muxed_video_audio_output.rtp")?;

    compare_video_dumps(
        &output_dump_from_disk,
        &new_output_dump,
        &[Duration::from_millis(500), Duration::from_millis(2500)],
        20.0,
    )?;

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