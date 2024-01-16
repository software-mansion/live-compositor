use crate::error::OutputInitError;

use self::rtp::{RtpSender, RtpSenderOptions};

use super::structs::EncodedChunk;

pub mod rtp;

pub enum Output {
    Rtp(RtpSender),
}

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
