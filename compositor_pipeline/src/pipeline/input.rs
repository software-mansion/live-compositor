use crate::error::InputInitError;

use rtp::{RtpReceiver, RtpReceiverOptions};

use self::rtp::ChunkIter;

pub mod rtp;

pub enum Input {
    Rtp(RtpReceiver),
}

impl Input {
    pub fn new(options: InputOptions) -> Result<(Self, ChunkIter), InputInitError> {
        match options {
            InputOptions::Rtp(opts) => Ok(RtpReceiver::new(opts)
                .map(|(receiver, chunk_iter)| (Self::Rtp(receiver), chunk_iter))?),
        }
    }
}

pub enum InputOptions {
    Rtp(RtpReceiverOptions),
}
