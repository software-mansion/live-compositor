use crossbeam_channel::Receiver;

use crate::error::OutputInitError;

use self::rtp::{RtpSender, RtpSenderOptions};

use super::{structs::EncoderOutputEvent, Port};

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
        packets: Receiver<EncoderOutputEvent>,
    ) -> Result<(Self, Option<Port>), OutputInitError> {
        match options {
            OutputOptions::Rtp(options) => {
                let (sender, port) = rtp::RtpSender::new(options, packets)?;
                Ok((Self::Rtp(sender), port))
            }
        }
    }
}
