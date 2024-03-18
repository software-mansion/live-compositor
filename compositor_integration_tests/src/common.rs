use anyhow::{Context, Result};
use bytes::Bytes;
use std::{
    fs,
    path::{Path, PathBuf},
};

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
