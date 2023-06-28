use anyhow::Result;

pub struct Encoder {}

impl Encoder {
    pub fn new() -> Self {
        Self {}
    }
    pub fn encode<I: IntoIterator<Item = bytes::Bytes>>(
        &self,
        _frames: I,
    ) -> Result<Vec<bytes::Bytes>> {
        todo!()
    }
}
