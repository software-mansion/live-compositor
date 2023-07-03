use anyhow::Result;

pub struct RtpPacket {
    pub data: bytes::Bytes,
}

pub struct RtpPacker {}

impl RtpPacker {
    pub fn new() -> Self {
        Self {}
    }

    #[allow(dead_code)]
    pub fn pack<I: IntoIterator<Item = RtpPacket>>(
        &self,
        _frames: I,
    ) -> Result<Option<bytes::Bytes>> {
        todo!()
    }
}

pub struct RtpParser {}

impl RtpParser {
    pub fn new() -> Self {
        Self {}
    }
    pub fn parse(&self, _raw_data: bytes::Bytes) -> Result<Vec<RtpPacket>> {
        todo!()
    }
}
