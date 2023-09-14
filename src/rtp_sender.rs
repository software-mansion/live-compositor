use log::error;
use std::{path::PathBuf, sync::Arc};

use compositor_pipeline::pipeline::PipelineOutputReceiver;
use ffmpeg_next::{
    codec,
    format::{self, context::Output},
    Codec, Packet,
};

pub struct RtpSender {
    output_ctx: Output,
    pub(crate) port: u16,
    pub(crate) ip: Arc<str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Options {
    pub port: u16,
    pub ip: Arc<str>,
}

// TODO: unwraps

impl PipelineOutputReceiver for RtpSender {
    type Opts = Options;
    type Identifier = Options;

    fn new(options: Options, codec: Codec) -> Self {
        let port = options.port;
        let ip = options.ip.clone();

        let mut output_ctx = format::output_as(
            &PathBuf::from(format!(
                "rtp://{}:{}?rtcpport={}",
                options.ip, options.port, options.port
            )),
            "rtp",
        )
        .unwrap();

        let mut stream = output_ctx.add_stream(codec).unwrap();
        unsafe {
            (*(*stream.as_mut_ptr()).codecpar).codec_id = codec::Id::H264.into();
        }

        output_ctx.write_header().unwrap();

        Self {
            output_ctx,
            port,
            ip,
        }
    }

    fn send_packet(&mut self, packet: Packet) {
        if let Err(err) = packet.write_interleaved(&mut self.output_ctx) {
            error!("Failed to send rtp packets: {err}")
        }
    }

    fn identifier(&self) -> Self::Identifier {
        Options {
            port: self.port,
            ip: self.ip.clone(),
        }
    }
}
