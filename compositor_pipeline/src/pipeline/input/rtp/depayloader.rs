use rtp::{
    codecs::{h264::H264Packet, opus::OpusPacket},
    packetizer::Depacketizer,
};

use crate::pipeline::structs::{AudioCodec, EncodedChunk, EncodedChunkKind, VideoCodec};

use super::{DepayloadingError, RtpStream};

pub enum Depayloader {
    Video(VideoDepayloader),
    Audio(AudioDepayloader),
    VideoWithAudio {
        video: VideoDepayloader,
        video_payload_type: u8,
        audio: AudioDepayloader,
        audio_payload_type: u8,
    },
}

impl Depayloader {
    pub fn new(stream: &RtpStream) -> Self {
        match stream {
            RtpStream::Video(codec) => Depayloader::Video(VideoDepayloader::new(codec)),
            RtpStream::Audio(codec) => Depayloader::Audio(AudioDepayloader::new(codec)),
            RtpStream::VideoWithAudio {
                video_codec,
                video_payload_type,
                audio_codec,
                audio_payload_type,
                ..
            } => Depayloader::VideoWithAudio {
                video: VideoDepayloader::new(video_codec),
                video_payload_type: *video_payload_type,
                audio: AudioDepayloader::new(audio_codec),
                audio_payload_type: *audio_payload_type,
            },
        }
    }

    pub fn depayload(
        &mut self,
        packet: rtp::packet::Packet,
    ) -> Result<Option<EncodedChunk>, DepayloadingError> {
        match self {
            Depayloader::Video(video) => video.depayload(packet),
            Depayloader::Audio(audio) => audio.depayload(packet),
            Depayloader::VideoWithAudio {
                video,
                video_payload_type,
                audio,
                audio_payload_type,
            } => {
                let pty = packet.header.payload_type;
                if pty == *video_payload_type {
                    video.depayload(packet)
                } else if pty == *audio_payload_type {
                    audio.depayload(packet)
                } else {
                    Err(DepayloadingError::BadPayloadType(pty))
                }
            }
        }
    }
}

pub enum VideoDepayloader {
    H264(H264Packet),
}

impl VideoDepayloader {
    pub fn new(codec: &VideoCodec) -> Self {
        match codec {
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
                let h264_packet = depayloader.depacketize(&packet.payload)?;

                if h264_packet.is_empty() {
                    return Ok(None);
                }

                Ok(Some(EncodedChunk {
                    data: h264_packet,
                    pts: packet.header.timestamp as i64,
                    dts: None,
                    kind,
                }))
            }
        }
    }
}

pub enum AudioDepayloader {
    Opus(OpusPacket),
}

impl AudioDepayloader {
    pub fn new(codec: &AudioCodec) -> Self {
        match codec {
            AudioCodec::Opus => AudioDepayloader::Opus(OpusPacket),
        }
    }

    fn depayload(
        &mut self,
        packet: rtp::packet::Packet,
    ) -> Result<Option<EncodedChunk>, DepayloadingError> {
        match self {
            AudioDepayloader::Opus(depayloader) => {
                let kind = EncodedChunkKind::Audio(AudioCodec::Opus);
                let h264_packet = depayloader.depacketize(&packet.payload)?;

                if h264_packet.is_empty() {
                    return Ok(None);
                }

                Ok(Some(EncodedChunk {
                    data: h264_packet,
                    pts: packet.header.timestamp as i64,
                    dts: None,
                    kind,
                }))
            }
        }
    }
}
