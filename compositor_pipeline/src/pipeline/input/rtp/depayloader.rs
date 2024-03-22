use std::time::Duration;

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
    },
}

impl VideoDepayloader {
    pub fn new(options: &decoder::VideoDecoderOptions) -> Self {
        match options.codec {
            VideoCodec::H264 => VideoDepayloader::H264 {
                depayloader: H264Packet::default(),
                buffer: vec![],
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
            } => {
                let kind = EncodedChunkKind::Video(VideoCodec::H264);
                let h264_chunk = depayloader.depacketize(&packet.payload)?;

                if h264_chunk.is_empty() {
                    return Ok(None);
                }

                if !packet.header.marker {
                    // not at access unit border, buffer the packet
                    buffer.push(h264_chunk);
                    return Ok(None);
                }

                let mut data = buffer.concat();
                data.extend(h264_chunk);

                let new_chunk = EncodedChunk {
                    data: data.into(),
                    pts: Duration::from_secs_f64(packet.header.timestamp as f64 / 90000.0),
                    dts: None,
                    kind,
                };

                *buffer = Vec::new();

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
    Opus(OpusPacket),
}

impl AudioDepayloader {
    pub fn new(options: &decoder::AudioDecoderOptions) -> Result<Self, AudioDepayloaderNewError> {
        match options {
            decoder::AudioDecoderOptions::Opus(_) => Ok(AudioDepayloader::Opus(OpusPacket)),
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
            AudioDepayloader::Opus(depayloader) => {
                let kind = EncodedChunkKind::Audio(AudioCodec::Opus);
                let opus_packet = depayloader.depacketize(&packet.payload)?;

                if opus_packet.is_empty() {
                    return Ok(None);
                }

                Ok(Some(EncodedChunk {
                    data: opus_packet,
                    pts: Duration::from_secs_f64(packet.header.timestamp as f64 / 48000.0),
                    dts: None,
                    kind,
                }))
            }
        }
    }
}
