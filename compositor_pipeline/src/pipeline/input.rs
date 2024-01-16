use crate::error::InputInitError;

use rtp::{RtpReceiver, RtpReceiverOptions};

use self::rtp::ChunksReceiver;

use super::ExactPort;

pub mod rtp;

pub enum Input {
    Rtp(RtpReceiver),
}

impl Input {
    pub fn new(options: InputOptions) -> Result<(Self, ChunksReceiver, ExactPort), InputInitError> {
        match options {
            InputOptions::Rtp(opts) => Ok(RtpReceiver::new(opts).map(
                |(receiver, chunks_receiver, port)| (Self::Rtp(receiver), chunks_receiver, port),
            )?),
        }
    }
}

pub enum InputOptions {
    Rtp(RtpReceiverOptions),
}
