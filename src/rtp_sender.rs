use log::error;
use std::{path::PathBuf, sync::Arc};

use compositor_pipeline::pipeline::PipelineOutput;
use ffmpeg_next::{
    codec,
    format::{self, context::Output},
    Codec, Packet,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RtpSender {
    pub(crate) port: u16,
    pub(crate) ip: Arc<str>,
}

pub struct RtpContext(Output);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Options {
    pub port: u16,
    pub ip: Arc<str>,
}

// TODO: unwraps

impl PipelineOutput for RtpSender {
    type Opts = Options;
    type Context = RtpContext;

    fn new(options: Options, codec: Codec) -> (Self, RtpContext) {
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

        (Self { port, ip }, RtpContext(output_ctx))
    }

    fn send_packet(&self, context: &mut RtpContext, packet: Packet) {
        if let Err(err) = packet.write_interleaved(&mut context.0) {
            error!("Failed to send rtp packets: {err}")
        }
    }
}
