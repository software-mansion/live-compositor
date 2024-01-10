use compositor_render::OutputId;
use log::error;
use std::sync::Arc;

use crate::{
    error::OutputInitError,
    pipeline::structs::{Codec, EncodedChunk},
};

use rand::Rng;
use rtp::packetizer::Payloader;
use webrtc_util::Marshal;

#[derive(Debug)]
pub struct RtpSender {
    pub port: u16,
    pub ip: Arc<str>,
    sender_thread: Option<std::thread::JoinHandle<()>>,
}

pub struct RtpContext {
    ssrc: u32,
    next_sequence_number: u16,
    payloader: rtp::codecs::h264::H264Payloader,
    socket: std::net::UdpSocket,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RtpSenderOptions {
    pub port: u16,
    pub ip: Arc<str>,
    pub codec: Codec,
    pub output_id: OutputId,
}

impl RtpSender {
    pub fn new(
        options: RtpSenderOptions,
        packets: Box<dyn Iterator<Item = EncodedChunk> + Send>,
    ) -> Result<Self, OutputInitError> {
        if options.codec != Codec::H264 {
            return Err(OutputInitError::UnsupportedCodec(options.codec));
        }

        let mut rng = rand::thread_rng();
        let ssrc = rng.gen::<u32>();
        let next_sequence_number = rng.gen::<u16>();
        let payloader = rtp::codecs::h264::H264Payloader::default();

        let socket = std::net::UdpSocket::bind(std::net::SocketAddrV4::new(
            std::net::Ipv4Addr::UNSPECIFIED,
            0,
        ))?;

        socket.connect((options.ip.as_ref(), options.port))?;

        let mut ctx = RtpContext {
            ssrc,
            next_sequence_number,
            payloader,
            socket,
        };

        let sender_thread = std::thread::Builder::new()
            .name(format!("RTP sender for output {}", options.output_id))
            .spawn(move || {
                for packet in packets {
                    Self::send_data(&mut ctx, packet);
                }
            })
            .unwrap();

        Ok(Self {
            port: options.port,
            ip: options.ip,
            sender_thread: Some(sender_thread),
        })
    }

    /// this assumes, that a "packet" contains data about a single frame (access unit)
    fn send_data(context: &mut RtpContext, packet: EncodedChunk) {
        // TODO: check if this is h264
        let EncodedChunk { data, pts, .. } = packet;

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

impl Drop for RtpSender {
    fn drop(&mut self) {
        match self.sender_thread.take() {
            Some(handle) => handle.join().unwrap(),
            None => error!("RTP sender thread was already joined."),
        }
        println!("rtp cleaned up nicely")
    }
}
