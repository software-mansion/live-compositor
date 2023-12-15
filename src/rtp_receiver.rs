use std::thread;

use bytes::BytesMut;
use compositor_common::scene::InputId;
use compositor_pipeline::{
    error::CustomError,
    pipeline::{decoder::DecoderParameters, PipelineInput},
};
use log::{error, warn};
use smol::{
    channel::{bounded, Receiver, Sender},
    future, net,
};
use webrtc_util::Unmarshal;

pub struct RtpReceiver {
    receiver_thread: Option<thread::JoinHandle<()>>,
    should_close_tx: Sender<()>,
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

        let socket = future::block_on(net::UdpSocket::bind(net::SocketAddrV4::new(
            net::Ipv4Addr::UNSPECIFIED,
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
                )))
            })
            .unwrap();

        Ok((
            Self {
                port: opts.port,
                receiver_thread: Some(receiver_thread),
                should_close_tx,
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

impl RtpReceiver {
    async fn rtp_receiver(
        socket: net::UdpSocket,
        packets_tx: Sender<rtp::packet::Packet>,
        should_close_rx: Receiver<()>,
    ) {
        let mut buffer = BytesMut::zeroed(65536);

        loop {
            let received_bytes = future::or(
                async {
                    should_close_rx.recv().await.unwrap();
                    None
                },
                async {
                    let (len, _) = socket.recv_from(&mut buffer).await.unwrap();
                    Some(len)
                },
            )
            .await;

            let Some(n) = received_bytes else {
                return;
            };

            let packet = match rtp::packet::Packet::unmarshal(&mut &buffer[..n]) {
                // https://datatracker.ietf.org/doc/html/rfc5761#section-4
                //
                // Given these constraints, it is RECOMMENDED to follow the guidelines
                // in the RTP/AVP profile [7] for the choice of RTP payload type values,
                // with the additional restriction that payload type values in the range
                // 64-95 MUST NOT be used.
                Ok(packet)
                    if packet.header.payload_type < 64 || packet.header.payload_type > 95 =>
                {
                    packet
                }
                Ok(_) | Err(_) => {
                    if rtcp::packet::unmarshal(&mut &buffer[..n]).is_err() {
                        warn!("Received an unexpected packet, which is not recognized either as RTP or RTCP. Dropping.");
                    }

                    continue;
                }
            };

            packets_tx.send(packet).await.unwrap();
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
