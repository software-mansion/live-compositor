use std::{mem, time::Duration};

use bytes::Bytes;
use log::error;
use rtp::{
    codecs::{h264::H264Packet, opus::OpusPacket},
    packetizer::Depacketizer,
};

use crate::pipeline::{
    decoder,
    rtp::{AUDIO_PAYLOAD_TYPE, VIDEO_PAYLOAD_TYPE},
    structs::{AudioCodec, EncodedChunk, EncodedChunkKind, VideoCodec},
};

use super::{DepayloadingError, RtpStream};

#[derive(Debug, thiserror::Error)]
pub enum DepayloaderNewError {
    #[error(transparent)]
    Audio(#[from] AudioDepayloaderNewError),
}

pub(crate) struct Depayloader {
    /// (Depayloader, payload type)
    pub video: Option<VideoDepayloader>,
    pub audio: Option<AudioDepayloader>,
}

impl Depayloader {
    pub fn new(stream: &RtpStream) -> Result<Self, DepayloaderNewError> {
        let video = stream
            .video
            .as_ref()
            .map(|video| VideoDepayloader::new(&video.options));

        let audio = stream
            .audio
            .as_ref()
            .map(|audio| AudioDepayloader::new(&audio.options))
            .transpose()?;

        Ok(Self { video, audio })
    }

    pub fn depayload(
        &mut self,
        packet: rtp::packet::Packet,
    ) -> Result<Option<EncodedChunk>, DepayloadingError> {
        match packet.header.payload_type {
            VIDEO_PAYLOAD_TYPE => match self.video.as_mut() {
                Some(video_depayloader) => video_depayloader.depayload(packet),
                None => Err(DepayloadingError::BadPayloadType(
                    packet.header.payload_type,
                )),
            },
            AUDIO_PAYLOAD_TYPE => match self.audio.as_mut() {
                Some(audio_depayloader) => audio_depayloader.depayload(packet),
                None => Err(DepayloadingError::BadPayloadType(
                    packet.header.payload_type,
                )),
            },
            other => Err(DepayloadingError::BadPayloadType(other)),
        }
    }
}

pub enum VideoDepayloader {
    H264 {
        depayloader: H264Packet,
        buffer: Vec<Bytes>,
        previous_timestamp: u32,
        rollover_count: usize,
    },
}

impl VideoDepayloader {
    pub fn new(options: &decoder::VideoDecoderOptions) -> Self {
        match options.codec {
            VideoCodec::H264 => VideoDepayloader::H264 {
                depayloader: H264Packet::default(),
                buffer: vec![],
                previous_timestamp: 0,
                rollover_count: 0,
            },
        }
    }

    fn depayload(
        &mut self,
        packet: rtp::packet::Packet,
    ) -> Result<Option<EncodedChunk>, DepayloadingError> {
        match self {
            VideoDepayloader::H264 {
                depayloader,
                buffer,
                previous_timestamp,
                rollover_count,
            } => {
                let kind = EncodedChunkKind::Video(VideoCodec::H264);
                let h264_chunk = depayloader.depacketize(&packet.payload)?;

                if h264_chunk.is_empty() {
                    return Ok(None);
                }

                buffer.push(h264_chunk);
                if !packet.header.marker {
                    // the marker bit is set on the last packet of an access unit
                    return Ok(None);
                }

                let timestamp = timestamp_with_rollover(
                    rollover_count,
                    *previous_timestamp,
                    packet.header.timestamp,
                );

                *previous_timestamp = packet.header.timestamp;

                let new_chunk = EncodedChunk {
                    data: mem::take(buffer).concat().into(),
                    pts: Duration::from_secs_f64(timestamp as f64 / 90000.0),
                    dts: None,
                    kind,
                };

                Ok(Some(new_chunk))
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AudioDepayloaderNewError {
    #[error("Unsupported depayloader for provided decoder settings: {0:?}")]
    UnsupportedDepayloader(decoder::AudioDecoderOptions),
}

pub enum AudioDepayloader {
    Opus {
        depayloader: OpusPacket,
        previous_timestamp: u32,
        rollover_count: usize,
    },
}

impl AudioDepayloader {
    pub fn new(options: &decoder::AudioDecoderOptions) -> Result<Self, AudioDepayloaderNewError> {
        match options {
            decoder::AudioDecoderOptions::Opus(_) => Ok(AudioDepayloader::Opus {
                depayloader: OpusPacket,
                previous_timestamp: 0,
                rollover_count: 0,
            }),
            decoder::AudioDecoderOptions::Aac(_) => Err(
                AudioDepayloaderNewError::UnsupportedDepayloader(options.clone()),
            ),
        }
    }

    fn depayload(
        &mut self,
        packet: rtp::packet::Packet,
    ) -> Result<Option<EncodedChunk>, DepayloadingError> {
        match self {
            AudioDepayloader::Opus {
                depayloader,
                previous_timestamp,
                rollover_count,
            } => {
                let kind = EncodedChunkKind::Audio(AudioCodec::Opus);
                let opus_packet = depayloader.depacketize(&packet.payload)?;

                if opus_packet.is_empty() {
                    return Ok(None);
                }

                let timestamp = timestamp_with_rollover(
                    rollover_count,
                    *previous_timestamp,
                    packet.header.timestamp,
                );

                *previous_timestamp = packet.header.timestamp;

                Ok(Some(EncodedChunk {
                    data: opus_packet,
                    pts: Duration::from_secs_f64(timestamp as f64 / 48000.0),
                    dts: None,
                    kind,
                }))
            }
        }
    }
}

fn timestamp_with_rollover(
    rollover_count: &mut usize,
    previous_timestamp: u32,
    current_timestamp: u32,
) -> u64 {
    let timestamp_diff = u32::abs_diff(previous_timestamp, current_timestamp);
    if timestamp_diff >= u32::MAX / 2 {
        if previous_timestamp > current_timestamp {
            *rollover_count += 1;
        } else {
            // We received a packet from before the rollover, so we need to decrement the count
            *rollover_count = rollover_count.saturating_sub(1);
        }
    }

    (*rollover_count as u64) * (u32::MAX as u64 + 1) + current_timestamp as u64
}

#[cfg(test)]
mod tests {
    use crate::pipeline::input::rtp::depayloader::timestamp_with_rollover;

    #[test]
    fn timestamp_rollover() {
        let mut rollover_count = 0;

        let previous_timestamp = 0;
        let current_timestamp = 1;
        assert_eq!(
            timestamp_with_rollover(&mut rollover_count, previous_timestamp, current_timestamp),
            1
        );

        let previous_timestamp = u32::MAX;
        let current_timestamp = 0;
        assert_eq!(
            timestamp_with_rollover(&mut rollover_count, previous_timestamp, current_timestamp),
            u32::MAX as u64 + 1 + current_timestamp as u64
        );

        let previous_timestamp = u32::MAX;
        let current_timestamp = 1;
        assert_eq!(
            timestamp_with_rollover(&mut rollover_count, previous_timestamp, current_timestamp),
            2 * (u32::MAX as u64 + 1) + current_timestamp as u64
        );

        let previous_timestamp = 1;
        let current_timestamp = u32::MAX;
        assert_eq!(
            timestamp_with_rollover(&mut rollover_count, previous_timestamp, current_timestamp),
            u32::MAX as u64 + 1 + current_timestamp as u64
        );

        let previous_timestamp = u32::MAX;
        let current_timestamp = u32::MAX - 1;
        assert_eq!(
            timestamp_with_rollover(&mut rollover_count, previous_timestamp, current_timestamp),
            u32::MAX as u64 + 1 + current_timestamp as u64
        );

        let previous_timestamp = u32::MAX - 1;
        let current_timestamp = u32::MAX;
        assert_eq!(
            timestamp_with_rollover(&mut rollover_count, previous_timestamp, current_timestamp),
            u32::MAX as u64 + 1 + current_timestamp as u64
        );
    }
}
