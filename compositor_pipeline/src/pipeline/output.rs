use crate::error::OutputInitError;

use self::rtp::{RtpSender, RtpSenderOptions};

use super::structs::EncodedChunk;

pub mod rtp;

#[derive(Debug)]
pub enum Output {
    Rtp(RtpSender),
}

#[derive(Debug, Clone)]
pub enum OutputOptions {
    Rtp(RtpSenderOptions),
}

impl Output {
    pub fn new(
        options: OutputOptions,
        packets: Box<dyn Iterator<Item = EncodedChunk> + Send>,
    ) -> Result<Self, OutputInitError> {
        match options {
            OutputOptions::Rtp(options) => {
                let sender = rtp::RtpSender::new(options, packets)?;
                Ok(Self::Rtp(sender))
            }
        }
    }
}
