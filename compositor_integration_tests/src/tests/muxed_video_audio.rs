use anyhow::Result;
use serde_json::json;
use std::time::Duration;

use crate::{
    audio_decoder::AudioChannels, compare_audio_dumps, compare_video_dumps, input_dump_from_disk,
    output_dump_from_disk, CommunicationProtocol, CompositorInstance, OutputReceiver, PacketSender,
};

pub fn muxed_video_audio() -> Result<()> {
    let instance = CompositorInstance::start();
    let input_port = instance.get_port();
    let output_port = instance.get_port();

    let output_receiver = OutputReceiver::start(
        output_port,
        CommunicationProtocol::Udp,
        Duration::from_secs(20),
        "muxed_video_audio_output.rtp",
    )?;

    instance.send_request(json!({
        "type": "register",
        "entity_type": "output_stream",
        "output_id": "output_1",
        "transport_protocol": "udp",
        "ip": "127.0.0.1",
        "port": output_port,
        "video": {
            "resolution": {
                "width": 640,
                "height": 360,
            },
            "encoder_preset": "ultrafast",
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
        "port": input_port,
        "video": {
            "codec": "h264"
        },
        "audio": {
            "codec": "opus"
        }
    }))?;

    let packets_dump = input_dump_from_disk("8_colors_input_video_audio.rtp")?;
    let mut packet_sender = PacketSender::new(CommunicationProtocol::Tcp, input_port)?;
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
