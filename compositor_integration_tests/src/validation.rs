use anyhow::{Context, Result};
use bytes::Bytes;
use compositor_render::Frame;
use rtp::packet::Packet;
use std::time::Duration;
use webrtc_util::Unmarshal;

use crate::video::VideoDecoder;

pub fn compare_dumps(
    expected: &Bytes,
    actual: &Bytes,
    timestamps: &[Duration],
    allowed_error: f32,
) -> Result<()> {
    let expected_packets = unmarshal_packets(expected)?;
    let actual_packets = unmarshal_packets(actual)?;

    let payload_type = expected_packets
        .first()
        .map(|p| p.header.payload_type)
        .context("No packets found")?;

    match payload_type {
        // Video H264
        96 => compare_video_dumps(expected_packets, actual_packets, timestamps, allowed_error)?,
        // Audio Opus
        97 => todo!(),
        _ => return Err(anyhow::anyhow!("Unsupported payload type")),
    }

    Ok(())
}

fn compare_video_dumps(
    expected: Vec<Packet>,
    actual: Vec<Packet>,
    timestamps: &[Duration],
    allowed_error: f32,
) -> Result<()> {
    let mut expected_video_decoder = VideoDecoder::new()?;
    let mut actual_video_decoder = VideoDecoder::new()?;

    for packet in expected {
        expected_video_decoder.decode(packet)?;
    }
    for packet in actual {
        actual_video_decoder.decode(packet)?;
    }

    let expected_frames = expected_video_decoder.take_frames()?;
    let actual_frames = actual_video_decoder.take_frames()?;

    for pts in timestamps {
        let expected = find_frame_for_pts(&expected_frames, pts)?;
        let actual = find_frame_for_pts(&actual_frames, pts)?;

        let diff_y = calculate_diff(&expected.data.y_plane, &actual.data.y_plane);
        let diff_u = calculate_diff(&expected.data.u_plane, &actual.data.u_plane);
        let diff_v = calculate_diff(&expected.data.v_plane, &actual.data.v_plane);

        if diff_y > allowed_error || diff_u > allowed_error || diff_v > allowed_error {
            let pts = pts.as_micros();
            return Err(anyhow::anyhow!(
                "Frame mismatch. PTS: {pts}, Diff Y: {diff_y}, Diff U: {diff_u}, Diff V: {diff_v}"
            ));
        }
    }

    Ok(())
}

fn unmarshal_packets(data: &Bytes) -> Result<Vec<Packet>> {
    let mut packets = Vec::new();
    let mut read_bytes = 0;
    while read_bytes < data.len() {
        let packet_size = u16::from_be_bytes([data[read_bytes], data[read_bytes + 1]]) as usize;
        read_bytes += 2;

        if data.len() < read_bytes + packet_size {
            break;
        }

        // TODO(noituri): Goodbye packet
        let packet = Packet::unmarshal(&mut &data[read_bytes..(read_bytes + packet_size)])?;
        read_bytes += packet_size;

        packets.push(packet);
    }

    Ok(packets)
}

fn calculate_diff(expected: &[u8], actual: &[u8]) -> f32 {
    if expected.len() != actual.len() {
        return f32::MAX;
    }

    let squere_error: f32 = expected
        .iter()
        .zip(actual.iter())
        .map(|(e, a)| (*e as i32 - *a as i32).pow(2) as f32)
        .sum();

    squere_error / expected.len() as f32
}

fn find_frame_for_pts(frames: &[Frame], pts: &Duration) -> Result<Frame> {
    frames
        .iter()
        .min_by_key(|f| u128::abs_diff(f.pts.as_micros(), pts.as_micros()))
        .cloned()
        .context("No frame found")
}
