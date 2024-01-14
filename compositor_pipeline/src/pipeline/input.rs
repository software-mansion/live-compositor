use crate::error::InputInitError;

use rtp::{RtpReceiver, RtpReceiverOptions};

use self::rtp::ChunksReceiver;

pub mod rtp;

pub enum Input {
    Rtp(RtpReceiver),
}

impl Input {
    pub fn new(options: InputOptions) -> Result<(Self, ChunksReceiver), InputInitError> {
        match options {
            InputOptions::Rtp(opts) => Ok(RtpReceiver::new(opts)
                .map(|(receiver, chunks_receiver)| (Self::Rtp(receiver), chunks_receiver))?),
        }
    }
}

pub enum InputOptions {
    Rtp(RtpReceiverOptions),
}
