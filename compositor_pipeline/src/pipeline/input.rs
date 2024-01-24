use crate::error::InputInitError;

use crossbeam_channel::Receiver;
use rtp::{RtpReceiver, RtpReceiverOptions};

use self::mp4::{Mp4, Mp4Options};

use super::{structs::EncodedChunk, Port};

pub mod mp4;
pub mod rtp;

pub enum Input {
    Rtp(RtpReceiver),
    Mp4(Mp4),
}

impl Input {
    pub fn new(
        options: InputOptions,
    ) -> Result<(Self, ChunksReceiver, Option<Port>), InputInitError> {
        match options {
            InputOptions::Rtp(opts) => Ok(RtpReceiver::new(opts).map(
                |(receiver, chunks_receiver, port)| {
                    (Self::Rtp(receiver), chunks_receiver, Some(port))
                },
            )?),

            InputOptions::Mp4(opts) => Ok(Mp4::new(opts)
                .map(|(mp4, chunks_receiver)| (Self::Mp4(mp4), chunks_receiver, None))?),
        }
    }
}

pub enum InputOptions {
    Rtp(RtpReceiverOptions),
    Mp4(Mp4Options),
}

#[derive(Debug)]
pub struct ChunksReceiver {
    pub video: Option<Receiver<EncodedChunk>>,
    pub audio: Option<Receiver<EncodedChunk>>,
}
