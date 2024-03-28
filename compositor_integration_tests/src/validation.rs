use anyhow::{Context, Result};
use bytes::Bytes;
use compositor_render::Frame;
use pitch_detection::detector::{mcleod::McLeodDetector, PitchDetector};
use rtp::packet::Packet;
use std::{fs, ops::Range, path::PathBuf, time::Duration};
use video_compositor::config::config;
use webrtc_util::Unmarshal;

use crate::{
    audio_decoder::{AudioChannels, AudioDecoder, AudioSampleBatch},
    video_decoder::VideoDecoder,
};

pub fn compare_video_dumps(
    expected: &Bytes,
    actual: &Bytes,
    timestamps: &[Duration],
    allowed_error: f32,
) -> Result<()> {
    let expected_packets = unmarshal_packets(expected)?;
    let actual_packets = unmarshal_packets(actual)?;

    let expected_video_packets = find_packets_for_payload_type(&expected_packets, 96);
    let actual_video_packets = find_packets_for_payload_type(&actual_packets, 96);

    let mut expected_video_decoder = VideoDecoder::new()?;
    let mut actual_video_decoder = VideoDecoder::new()?;

    for packet in expected_video_packets {
        expected_video_decoder.decode(packet)?;
    }
    for packet in actual_video_packets {
        actual_video_decoder.decode(packet)?;
    }

    let expected_frames = expected_video_decoder.take_frames()?;
    let actual_frames = actual_video_decoder.take_frames()?;

    for pts in timestamps {
        let expected_frame = find_frame_for_pts(&expected_frames, pts)?;
        let actual_frame = find_frame_for_pts(&actual_frames, pts)?;

        let diff_y = calculate_mse(&expected_frame.data.y_plane, &actual_frame.data.y_plane);
        let diff_u = calculate_mse(&expected_frame.data.u_plane, &actual_frame.data.u_plane);
        let diff_v = calculate_mse(&expected_frame.data.v_plane, &actual_frame.data.v_plane);

        if diff_y > allowed_error || diff_u > allowed_error || diff_v > allowed_error {
            let pts = pts.as_micros();

            let mut expected_data = vec![];
            expected_data.extend_from_slice(&expected_frame.data.y_plane);
            expected_data.extend_from_slice(&expected_frame.data.u_plane);
            expected_data.extend_from_slice(&expected_frame.data.v_plane);

            let mut actual_data = vec![];
            actual_data.extend_from_slice(&actual_frame.data.y_plane);
            actual_data.extend_from_slice(&actual_frame.data.u_plane);
            actual_data.extend_from_slice(&actual_frame.data.v_plane);

            let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .join("failed_snapshot_tests");

            let _ = fs::create_dir_all(&path);
            fs::write(
                path.join(format!("expected_frame_{pts}.yuv")),
                expected_data,
            )
            .unwrap();
            fs::write(path.join(format!("actual_frame_{pts}.yuv")), actual_data).unwrap();
            fs::write(path.join(format!("expected_frame_{pts}.rtp")), expected).unwrap();
            fs::write(path.join(format!("actual_frame_{pts}.rtp")), actual).unwrap();

            return Err(anyhow::anyhow!(
                "Frame mismatch. PTS: {pts}, Diff Y: {diff_y}, Diff U: {diff_u}, Diff V: {diff_v}"
            ));
        }
    }

    Ok(())
}

pub fn compare_audio_dumps(
    expected: &Bytes,
    actual: &Bytes,
    sampling_intervals: &[Range<Duration>],
    allowed_error: f32,
    channels: AudioChannels,
) -> Result<()> {
    let expected_packets = unmarshal_packets(expected)?;
    let actual_packets = unmarshal_packets(actual)?;
    let expected_audio_packets = find_packets_for_payload_type(&expected_packets, 97);
    let actual_audio_packets = find_packets_for_payload_type(&actual_packets, 97);

    let sample_rate = config().output_sample_rate;

    let mut expected_audio_decoder = AudioDecoder::new(sample_rate, channels)?;
    let mut actual_audio_decoder = AudioDecoder::new(sample_rate, channels)?;

    for packet in expected_audio_packets {
        expected_audio_decoder.decode(packet)?;
    }
    for packet in actual_audio_packets {
        actual_audio_decoder.decode(packet)?;
    }

    let expected_samples = expected_audio_decoder.take_samples();
    let actual_samples = actual_audio_decoder.take_samples();

    for time_range in sampling_intervals {
        let expected = find_sample_batches(&expected_samples, time_range.clone());
        let actual = find_sample_batches(&actual_samples, time_range.clone());

        let (expected_pitch_left, expected_pitch_right) =
            pitch_from_sample_batch(expected, sample_rate)?;
        let (actual_pitch_left, actual_pitch_right) = pitch_from_sample_batch(actual, sample_rate)?;

        let diff_pitch_left = f64::abs(expected_pitch_left - actual_pitch_left);
        let diff_pitch_right = f64::abs(expected_pitch_right - actual_pitch_right);

        if diff_pitch_left > allowed_error as f64 || diff_pitch_right > allowed_error as f64 {
            let pts_start = time_range.start.as_micros();
            let pts_end = time_range.end.as_micros();

            return Err(anyhow::anyhow!(
                "Audio mismatch. Time range: ({pts_start}, {pts_end}), Diff Pitch Left: {diff_pitch_left}, Diff Pitch Right: {diff_pitch_right}"
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

fn calculate_mse(expected: &[u8], actual: &[u8]) -> f32 {
    if expected.len() != actual.len() {
        return f32::MAX;
    }

    let square_error: f32 = expected
        .iter()
        .zip(actual.iter())
        .map(|(e, a)| (*e as i32 - *a as i32).pow(2) as f32)
        .sum();

    square_error / expected.len() as f32
}

fn find_frame_for_pts(frames: &[Frame], pts: &Duration) -> Result<Frame> {
    frames
        .iter()
        .min_by_key(|f| u128::abs_diff(f.pts.as_micros(), pts.as_micros()))
        .cloned()
        .context("No frame found")
}

fn find_sample_batches(
    samples: &[AudioSampleBatch],
    time_range: Range<Duration>,
) -> Vec<AudioSampleBatch> {
    samples
        .iter()
        .filter(|s| time_range.contains(&s.pts))
        .cloned()
        .collect()
}

fn pitch_from_sample_batch(
    sample_batch: Vec<AudioSampleBatch>,
    sample_rate: u32,
) -> Result<(f64, f64)> {
    fn get_pitch(samples: &[f64], sample_rate: u32) -> Result<f64> {
        let mut detector: McLeodDetector<f64> = McLeodDetector::new(samples.len(), 0);
        detector
            .get_pitch(samples, sample_rate as usize, 0.0, 0.0)
            .context("No pitch found")
            .map(|pitch| pitch.frequency)
    }

    let left_samples = sample_batch
        .iter()
        .flat_map(|batch| &batch.samples)
        .step_by(2)
        .map(|sample| *sample as f64 / i16::MAX as f64)
        .collect::<Vec<_>>();

    let right_samples = sample_batch
        .iter()
        .flat_map(|batch| &batch.samples)
        .skip(1)
        .step_by(2)
        .map(|sample| *sample as f64 / i16::MAX as f64)
        .collect::<Vec<_>>();

    Ok((
        get_pitch(&left_samples, sample_rate)?,
        get_pitch(&right_samples, sample_rate)?,
    ))
}

fn find_packets_for_payload_type(packets: &[Packet], payload_type: u8) -> Vec<Packet> {
    packets
        .iter()
        .filter(|p| p.header.payload_type == payload_type)
        .cloned()
        .collect()
}
