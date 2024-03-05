use crossbeam_channel::Receiver;

use crate::{error::OutputInitError, queue::PipelineEvent};

use self::rtp::{RtpSender, RtpSenderOptions};

use super::{structs::EncodedChunk, Port};

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
        packets: Receiver<PipelineEvent<EncodedChunk>>,
    ) -> Result<(Self, Option<Port>), OutputInitError> {
        match options {
            OutputOptions::Rtp(options) => {
                let (sender, port) = rtp::RtpSender::new(options, packets)?;
                Ok((Self::Rtp(sender), port))
            }
        }
    }
}
