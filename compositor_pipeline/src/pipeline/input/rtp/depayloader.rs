use std::time::Duration;

use rtp::{
    codecs::{h264::H264Packet, opus::OpusPacket},
    packetizer::Depacketizer,
};

use crate::pipeline::{
    decoder,
    structs::{AudioCodec, EncodedChunk, EncodedChunkKind, VideoCodec},
};

use super::{DepayloadingError, RtpStream};

pub struct PayloadType(u8);

#[derive(Debug, thiserror::Error)]
pub enum DepayloaderNewError {
    #[error(transparent)]
    Audio(#[from] AudioDepayloaderNewError),
}

pub struct Depayloader {
    /// (Depayloader, payload type)
    pub video: Option<(VideoDepayloader, PayloadType)>,
    pub audio: Option<(AudioDepayloader, PayloadType)>,
}

impl Depayloader {
    pub fn new(stream: &RtpStream) -> Result<Self, DepayloaderNewError> {
        let video = stream.video.as_ref().map(|video| {
            (
                VideoDepayloader::new(&video.options),
                PayloadType(video.payload_type),
            )
        });

        let audio = match stream.audio.as_ref() {
            Some(audio) => Some((
                AudioDepayloader::new(&audio.options)?,
                PayloadType(audio.payload_type),
            )),

            None => None,
        };

        Ok(Self { video, audio })
    }

    pub fn depayload(
        &mut self,
        packet: rtp::packet::Packet,
    ) -> Result<Option<EncodedChunk>, DepayloadingError> {
        let pty = packet.header.payload_type;
        if let Some((video_depayloader, video_payload_type)) = &mut self.video {
            if video_payload_type.0 == pty {
                return video_depayloader.depayload(packet);
            }
        }

        if let Some((audio_depayloader, audio_payload_type)) = &mut self.audio {
            if audio_payload_type.0 == pty {
                return audio_depayloader.depayload(packet);
            }
        }

        Err(DepayloadingError::BadPayloadType(pty))
    }
}

pub enum VideoDepayloader {
    H264(H264Packet),
}

impl VideoDepayloader {
    pub fn new(options: &decoder::VideoDecoderOptions) -> Self {
        match options.codec {
            VideoCodec::H264 => VideoDepayloader::H264(H264Packet::default()),
        }
    }

    fn depayload(
        &mut self,
        packet: rtp::packet::Packet,
    ) -> Result<Option<EncodedChunk>, DepayloadingError> {
        match self {
            VideoDepayloader::H264(depayloader) => {
                let kind = EncodedChunkKind::Video(VideoCodec::H264);
                let h264_chunk = depayloader.depacketize(&packet.payload)?;

                if h264_chunk.is_empty() {
                    return Ok(None);
                }

                Ok(Some(EncodedChunk {
                    data: h264_chunk,
                    pts: Duration::from_secs_f64(packet.header.timestamp as f64 / 90000.0),
                    dts: None,
                    kind,
                }))
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
