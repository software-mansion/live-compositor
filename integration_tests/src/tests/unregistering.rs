use std::{thread, time::Duration};

use crate::{
    compare_video_dumps, input_dump_from_disk, CommunicationProtocol, CompositorInstance,
    OutputReceiver, PacketSender, VideoValidationConfig,
};
use anyhow::Result;
use serde_json::json;

/// Checks if input stream frames are not shown after unregistering.
///
/// Show image on the right side for 20 seconds.
#[test]
pub fn unregistering() -> Result<()> {
    const OUTPUT_DUMP_FILE: &str = "unregistering_test_output.rtp";
    let instance = CompositorInstance::start();
    let input_port = instance.get_port();
    let output_port = instance.get_port();

    // This should fail because image is not registered yet.
    register_output_with_initial_scene(&instance, output_port)
        .expect_err("Image has to be registered first");

    instance.send_request(json!({
        "type": "register",
        "entity_type": "image",
        "asset_type": "svg",
        "image_id": "image_1",
        "url": "https://compositor.live/img/logo.svg"
    }))?;

    register_output_with_initial_scene(&instance, output_port)?;

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
        "transport_protocol": "udp",
        "input_id": "input_1",
        "port": input_port,
        "video": {
            "codec": "h264"
        },
    }))?;

    let input_1_dump = input_dump_from_disk("8_colors_long_input_video.rtp")?;
    let mut input_1_sender = PacketSender::new(CommunicationProtocol::Udp, input_port)?;

    instance.send_request(json!({
        "type": "start",
    }))?;

    thread::sleep(Duration::from_secs(2));

    // After whole input dump is sent, unregister input stream immediately.
    input_1_sender.send(&input_1_dump)?;
    instance.send_request(json!({
        "type": "unregister",
        "entity_type": "input_stream",
        "input_id": "input_1",
    }))?;

    instance.send_request(json!({
        "type": "unregister",
        "entity_type": "image",
        "image_id": "image_1",
    }))?;

    let new_output_dump = output_receiver.wait_for_output()?;

    compare_video_dumps(
        OUTPUT_DUMP_FILE,
        &new_output_dump,
        VideoValidationConfig {
            allowed_invalid_frames: 1,
            ..Default::default()
        },
    )?;

    Ok(())
}

fn register_output_with_initial_scene(instance: &CompositorInstance, port: u16) -> Result<()> {
    instance.send_request(json!({
        "type": "register",
        "entity_type": "output_stream",
        "output_id": "output_1",
        "transport_protocol": "tcp_server",
        "port": port,
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
                        "type": "image",
                        "image_id": "image_1",
                    },
                ],
            }
        },
    }))
}
