use anyhow::Result;

pub struct RtpFrame {
    pub data: bytes::Bytes,
}

pub struct RtpPacker {}

impl RtpPacker {
    pub fn new() -> Self {
        Self {}
    }

    #[allow(dead_code)]
    pub fn pack<I: IntoIterator<Item = RtpFrame>>(
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
    pub fn parse(&self, _raw_data: bytes::Bytes) -> Result<Vec<RtpFrame>> {
        todo!()
    }
}
