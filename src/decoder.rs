use anyhow::Result;

pub struct Decoder {}

impl Decoder {
    pub fn new() -> Self {
        Decoder {}
    }

    pub fn decode(&self, _buffer: bytes::Bytes) -> Result<Vec<bytes::Bytes>> {
        todo!()
    }
}
