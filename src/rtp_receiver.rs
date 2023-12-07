use std::{mem, thread};

use bytes::BytesMut;
use compositor_common::scene::InputId;
use compositor_pipeline::{
    error::CustomError,
    pipeline::{decoder::DecoderParameters, PipelineInput},
};
use log::{error, warn};
use smol::{
    channel::{bounded, Receiver, SendError, Sender},
    future, net,
};
use webrtc_util::Unmarshal;

pub struct RtpReceiver {
    receiver_thread: Option<thread::JoinHandle<()>>,
    should_close_tx: Sender<()>,
    /// sender channel to register EOS listener callbacks
    eos_listener_subscriber_tx: Sender<Box<dyn FnOnce() + Send>>,
    pub port: u16,
}

pub struct Options {
    pub port: u16,
    pub input_id: InputId,
}

impl PipelineInput for RtpReceiver {
    type Opts = Options;
    type PacketIterator = PacketIter;

    fn new(opts: Self::Opts) -> Result<(Self, Self::PacketIterator), CustomError> {
        let (should_close_tx, should_close_rx) = bounded(1);
        let (packets_tx, packets_rx) = bounded(1);
        let (eos_listener_subscriber_tx, eos_listener_subscriber_rx) = bounded(1);

        let socket = future::block_on(net::UdpSocket::bind(net::SocketAddrV4::new(
            net::Ipv4Addr::LOCALHOST,
            opts.port,
        )))
        .map_err(|e| CustomError(Box::new(e)))?;

        let receiver_thread = thread::Builder::new()
            .name(format!("RTP receiver {}", opts.input_id))
            .spawn(move || {
                let executor = smol::LocalExecutor::new();

                future::block_on(executor.run(RtpReceiver::rtp_receiver(
                    socket,
                    packets_tx,
                    should_close_rx,
                    eos_listener_subscriber_rx,
                )))
            })
            .unwrap();

        Ok((
            Self {
                port: opts.port,
                receiver_thread: Some(receiver_thread),
                should_close_tx,
                eos_listener_subscriber_tx,
            },
            PacketIter {
                receiver: packets_rx,
            },
        ))
    }

    fn decoder_parameters(&self) -> DecoderParameters {
        DecoderParameters {
            codec: compositor_pipeline::pipeline::decoder::Codec::H264,
        }
    }
}

enum ThreadMessage {
    Data(usize),
    Abort,
    Skip,
}

impl RtpReceiver {
    pub fn subscribe_eos_listener(
        &self,
        callback: Box<dyn FnOnce() + Send>,
    ) -> Result<(), SendError<Box<dyn FnOnce() + Send>>> {
        self.eos_listener_subscriber_tx.send_blocking(callback)
    }

    async fn rtp_receiver(
        socket: net::UdpSocket,
        packets_tx: Sender<rtp::packet::Packet>,
        should_close_rx: Receiver<()>,
        eos_listener_subscriber_rx: Receiver<Box<dyn FnOnce() + Send>>,
    ) {
        let mut buffer = BytesMut::zeroed(65536);
        let mut eos_listeners = vec![];

        loop {
            let msg = future::or(
                async {
                    should_close_rx.recv().await.unwrap();
                    ThreadMessage::Abort
                },
                future::or(
                    async {
                        let eos_listener = eos_listener_subscriber_rx.recv().await.unwrap();
                        eos_listeners.push(eos_listener);
                        ThreadMessage::Skip
                    },
                    async {
                        let (len, _) = socket.recv_from(&mut buffer).await.unwrap();
                        ThreadMessage::Data(len)
                    },
                ),
            )
            .await;

            let bytes_len = match msg {
                ThreadMessage::Data(len) => len,
                ThreadMessage::Abort => return,
                ThreadMessage::Skip => {
                    continue;
                }
            };

            Self::handle_rtp_packet(&mut buffer, bytes_len, &packets_tx, &mut eos_listeners).await
        }
    }

    async fn handle_rtp_packet(
        buffer: &mut [u8],
        bytes_len: usize,
        packets_tx: &Sender<rtp::packet::Packet>,
        eos_listeners: &mut Vec<Box<dyn FnOnce() + Send>>,
    ) {
        match rtp::packet::Packet::unmarshal(&mut &buffer[..bytes_len]) {
            // https://datatracker.ietf.org/doc/html/rfc5761#section-4
            //
            // Given these constraints, it is RECOMMENDED to follow the guidelines
            // in the RTP/AVP profile [7] for the choice of RTP payload type values,
            // with the additional restriction that payload type values in the range
            // 64-95 MUST NOT be used.
            Ok(packet) if packet.header.payload_type < 64 || packet.header.payload_type > 95 => {
                if let Err(err) = packets_tx.send(packet).await {
                    error!("Sending to closed channel in RTP receiver. {err}")
                }
            }
            Ok(_) | Err(_) => match rtcp::packet::unmarshal(&mut &buffer[..bytes_len]) {
                Ok(packets) => Self::handle_rtcp_packets(packets, eos_listeners).await,
                Err(_) => {
                    warn!("Received an unexpected packet, which is not recognized either as RTP or RTCP. Dropping.");
                }
            },
        };
    }

    async fn handle_rtcp_packets(
        packets: Vec<Box<dyn rtcp::packet::Packet + Send + Sync>>,
        eos_listeners: &mut Vec<Box<dyn FnOnce() + Send>>,
    ) {
        for packet in packets.iter() {
            if packet.header().packet_type == rtcp::header::PacketType::Goodbye {
                for callback in mem::take(eos_listeners).into_iter() {
                    callback();
                }
            }
        }
    }
}

impl Drop for RtpReceiver {
    fn drop(&mut self) {
        self.should_close_tx.send_blocking(()).unwrap();
        if let Some(thread) = self.receiver_thread.take() {
            thread.join().unwrap();
        } else {
            error!("RTP receiver does not hold a thread handle to the receiving thread.")
        }
    }
}

pub struct PacketIter {
    receiver: Receiver<rtp::packet::Packet>,
}

impl Iterator for PacketIter {
    type Item = rtp::packet::Packet;

    fn next(&mut self) -> Option<Self::Item> {
        self.receiver.recv_blocking().ok()
    }
}
