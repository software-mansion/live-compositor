use bytes::Bytes;
use log::{error, warn};
use std::sync::Arc;

use compositor_pipeline::{error::CustomError, pipeline::PipelineOutput};
use ffmpeg_next::{codec, Codec, Packet};

use rand::Rng;
use rtp::packetizer::Payloader;
use webrtc_util::Marshal;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RtpSender {
    pub(crate) port: u16,
    pub(crate) ip: Arc<str>,
}

pub struct RtpContext {
    ssrc: u32,
    next_sequence_number: u16,
    payloader: rtp::codecs::h264::H264Payloader,
    socket: std::net::UdpSocket,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Options {
    pub port: u16,
    pub ip: Arc<str>,
}

impl PipelineOutput for RtpSender {
    type Opts = Options;
    type Context = RtpContext;

    fn new(options: Options, codec: Codec) -> Result<(Self, RtpContext), CustomError> {
        if codec.id() != codec::Id::H264 {
            unimplemented!("Only H264 is supported");
        }

        let mut rng = rand::thread_rng();
        let ssrc = rng.gen::<u32>();
        let next_sequence_number = rng.gen::<u16>();
        let payloader = rtp::codecs::h264::H264Payloader::default();

        let socket = std::net::UdpSocket::bind("0.0.0.0:0").map_err(|e| CustomError(e.into()))?;
        socket
            .connect((options.ip.as_ref(), options.port))
            .map_err(|e| CustomError(e.into()))?;

        Ok((
            Self {
                port: options.port,
                ip: options.ip,
            },
            RtpContext {
                ssrc,
                next_sequence_number,
                payloader,
                socket,
            },
        ))
    }

    /// this assumes, that a "packet" contains data about a single frame (access unit)
    fn send_packet(&self, context: &mut RtpContext, packet: Packet) {
        let Some(data) = packet.data() else {
            warn!("No data is present in a packet received from the encoder");
            return;
        };

        let Some(pts) = packet.pts() else {
            warn!("No pts is present in a packet received from the encoder");
            return;
        };

        let data = Bytes::copy_from_slice(data);
        let payloads = match context.payloader.payload(1500, &data) {
            Ok(p) => p,
            Err(e) => {
                error!("Failed to payload a packet: {}", e);
                return;
            }
        };
        let packets_amount = payloads.len();

        for (i, payload) in payloads.into_iter().enumerate() {
            let header = rtp::header::Header {
                version: 2,
                padding: false,
                extension: false,
                marker: i == packets_amount - 1, // marker needs to be set on the last packet of each frame
                payload_type: 96,
                sequence_number: context.next_sequence_number,
                timestamp: pts as u32,
                ssrc: context.ssrc,
                ..Default::default()
            };

            let packet = rtp::packet::Packet { header, payload };

            let packet = match packet.marshal() {
                Ok(p) => p,
                Err(e) => {
                    error!("Failed to marshal a packet: {}", e);
                    return;
                }
            };

            if let Err(err) = context.socket.send(&packet) {
                error!("Failed to send packet: {err}");
            }

            context.next_sequence_number = context.next_sequence_number.wrapping_add(1);
        }
    }
}
