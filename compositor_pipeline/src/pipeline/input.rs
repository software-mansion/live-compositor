use crate::{error::InputInitError, pipeline::structs::EncodedChunk};

use rtp::{RtpReceiver, RtpReceiverOptions};

pub mod rtp;

pub enum Input {
    Rtp(RtpReceiver),
}

impl Input {
    pub fn new(
        options: InputOptions,
    ) -> Result<(Self, Box<dyn Iterator<Item = EncodedChunk> + Send>), InputInitError> {
        match options {
            InputOptions::Rtp(opts) => Ok(RtpReceiver::new(opts).map(|(receiver, iter)| {
                (
                    Self::Rtp(receiver),
                    Box::new(iter) as Box<dyn Iterator<Item = EncodedChunk> + Send>,
                )
            })?),
        }
    }
}

pub enum InputOptions {
    Rtp(RtpReceiverOptions),
}
