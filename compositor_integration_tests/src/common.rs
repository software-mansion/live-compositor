use anyhow::{Context, Result};
use bytes::Bytes;
use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};
use webrtc_util::Unmarshal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommunicationProtocol {
    Udp,
    Tcp,
}

pub fn input_dump_from_disk<P: AsRef<Path>>(path: P) -> Result<Bytes> {
    let input_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("snapshot_tests")
        .join("snapshots")
        .join("rtp_packet_dumps")
        .join("inputs")
        .join(path);

    let bytes = fs::read(input_path).context("Failed to read input dump")?;
    Ok(Bytes::from(bytes))
}

pub fn output_dump_from_disk<P: AsRef<Path>>(path: P) -> Result<Bytes> {
    let output_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("snapshot_tests")
        .join("snapshots")
        .join("rtp_packet_dumps")
        .join("outputs")
        .join(path);

    let bytes = fs::read(output_path).context("Failed to read output dump")?;
    Ok(Bytes::from(bytes))
}

pub fn split_rtp_packet_dump(dump: Bytes, split_at_pts: Duration) -> Result<(Bytes, Bytes)> {
    let mut read_bytes = 0;
    let mut start_pts = None;

    while read_bytes < dump.len() {
        let packet_len = u16::from_be_bytes([dump[read_bytes], dump[read_bytes + 1]]) as usize;
        read_bytes += 2;

        let packet =
            rtp::packet::Packet::unmarshal(&mut dump.slice(read_bytes..(read_bytes + packet_len)))?;
        read_bytes += packet_len;

        let packet_pts = match packet.header.payload_type {
            96 => Duration::from_secs_f64(packet.header.timestamp as f64 / 90000.0),
            97 => Duration::from_secs_f64(packet.header.timestamp as f64 / 48000.0),
            payload_type => {
                return Err(anyhow::anyhow!("Unsupported payload type: {payload_type}"))
            }
        };

        let start_pts = start_pts.get_or_insert(packet_pts);
        if packet_pts.as_micros() - start_pts.as_micros() >= split_at_pts.as_micros() {
            return Ok((dump.slice(0..read_bytes), dump.slice(read_bytes..)));
        }
    }

    Ok((dump, Bytes::new()))
}
