use rtp::{
    codecs::{h264::H264Packet, opus::OpusPacket},
    packetizer::Depacketizer,
};

use crate::pipeline::structs::{AudioCodec, EncodedChunk, EncodedChunkKind, VideoCodec};

use super::{DepayloadingError, RtpStream};

pub struct Depayloader {
    /// (Depayloader, payload type)
    pub video: Option<(VideoDepayloader, u8)>,
    pub audio: Option<(AudioDepayloader, u8)>,
}

impl Depayloader {
    pub fn new(stream: &RtpStream) -> Self {
        let video = stream
            .video
            .as_ref()
            .map(|video| (VideoDepayloader::new(&video.codec), video.payload_type));
        let audio = stream
            .audio
            .as_ref()
            .map(|audio| (AudioDepayloader::new(&audio.codec), audio.payload_type));

        Self { video, audio }
    }

    pub fn depayload(
        &mut self,
        packet: rtp::packet::Packet,
    ) -> Result<Option<EncodedChunk>, DepayloadingError> {
        let pty = packet.header.payload_type;
        if let Some((video_depayloader, video_payload_type)) = &mut self.video {
            if *video_payload_type == pty {
                return video_depayloader.depayload(packet);
            }
        }

        if let Some((audio_depayloader, audio_payload_type)) = &mut self.audio {
            if *audio_payload_type == pty {
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
