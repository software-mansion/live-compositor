use compositor_render::OutputId;
use log::{debug, error};
use std::{
    io::{self, Write},
    sync::{atomic::AtomicBool, Arc},
    thread,
    time::Duration,
    u16,
};

use rand::Rng;
use rtp::packetizer::Payloader;
use webrtc_util::Marshal;

use crate::{
    error::OutputInitError,
    pipeline::{
        rtp::{bind_to_requested_port, BindToPortError, RequestedPort},
        structs::EncodedChunk,
        Port,
    },
};

#[derive(Debug)]
pub struct RtpSender {
    pub connection_options: RtpConnectionOptions,
    sender_thread: Option<std::thread::JoinHandle<()>>,
    should_close: Arc<AtomicBool>,
}

struct RtpContext {
    ssrc: u32,
    next_sequence_number: u16,
    payloader: rtp::codecs::h264::H264Payloader,
    socket: socket2::Socket,
    should_close: Arc<AtomicBool>,
}

#[derive(Debug, Clone)]
pub struct RtpSenderOptions {
    pub output_id: OutputId,
    pub connection_options: RtpConnectionOptions,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RtpConnectionOptions {
    Udp { port: Port, ip: Arc<str> },
    TcpServer { port: RequestedPort },
}

impl RtpSender {
    pub fn new(
        options: RtpSenderOptions,
        packets: Box<dyn Iterator<Item = EncodedChunk> + Send>,
    ) -> Result<(Self, Option<Port>), OutputInitError> {
        let mut rng = rand::thread_rng();
        let ssrc = rng.gen::<u32>();
        let next_sequence_number = rng.gen::<u16>();
        let payloader = rtp::codecs::h264::H264Payloader::default();

        let (socket, port) = match &options.connection_options {
            RtpConnectionOptions::Udp { port, ip } => Self::udp_socket(ip, *port)?,
            RtpConnectionOptions::TcpServer { port } => Self::tcp_socket(*port)?,
        };

        let should_close = Arc::new(AtomicBool::new(false));
        let mut ctx = RtpContext {
            ssrc,
            next_sequence_number,
            payloader,
            socket: socket.try_clone()?,
            should_close: should_close.clone(),
        };

        let connection_options = options.connection_options.clone();
        let sender_thread = std::thread::Builder::new()
            .name(format!("RTP sender for output {}", options.output_id))
            .spawn(move || match connection_options {
                RtpConnectionOptions::Udp { .. } => run_udp_sender_thread(&mut ctx, packets),
                RtpConnectionOptions::TcpServer { .. } => run_tcp_sender_thread(&mut ctx, packets),
            })
            .unwrap();

        Ok((
            Self {
                connection_options: options.connection_options,
                sender_thread: Some(sender_thread),
                should_close,
            },
            Some(port),
        ))
    }

    fn udp_socket(ip: &str, port: Port) -> Result<(socket2::Socket, Port), OutputInitError> {
        let socket = std::net::UdpSocket::bind(std::net::SocketAddrV4::new(
            std::net::Ipv4Addr::UNSPECIFIED,
            0,
        ))?;

        socket.connect((ip, port.0))?;
        Ok((socket.into(), port))
    }

    fn tcp_socket(port: RequestedPort) -> Result<(socket2::Socket, Port), OutputInitError> {
        let socket = socket2::Socket::new(
            socket2::Domain::IPV4,
            socket2::Type::STREAM,
            Some(socket2::Protocol::TCP),
        )
        .map_err(OutputInitError::SocketError)?;

        let port = bind_to_requested_port(port, &socket)?;

        socket.listen(1).map_err(OutputInitError::SocketError)?;
        Ok((socket, port))
    }
}

fn run_tcp_sender_thread(
    context: &mut RtpContext,
    mut packets: Box<dyn Iterator<Item = EncodedChunk> + Send>,
) {
    // make accept non blocking so we have a chance to handle should_close value
    context
        .socket
        .set_nonblocking(true)
        .expect("Cannot set non-blocking");
    loop {
        if context
            .should_close
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            return;
        }

        // accept only one connection at the time
        let Ok((socket, _)) = context.socket.accept() else {
            thread::sleep(Duration::from_millis(50));
            continue;
        };

        let mut socket = TcpWritePacketStream::new(socket, context.should_close.clone());
        loop {
            let Some(EncodedChunk { data, pts, .. }) = packets.next() else {
                return;
            };

            let payloads = match context.payloader.payload(64000, &data) {
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
                    timestamp: (pts.as_secs_f64() * 90000.0) as u32,
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

                if let Err(err) = socket.write_packet(packet) {
                    debug!("Failed to send packet: {err}");
                }

                context.next_sequence_number = context.next_sequence_number.wrapping_add(1);
            }
        }
    }
}

/// this assumes, that a "packet" contains data about a single frame (access unit)
fn run_udp_sender_thread(
    context: &mut RtpContext,
    packets: Box<dyn Iterator<Item = EncodedChunk> + Send>,
) {
    for packet in packets {
        // TODO: check if this is h264
        let EncodedChunk { data, pts, .. } = packet;

        let payloads = match context.payloader.payload(1400, &data) {
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
                timestamp: (pts.as_secs_f64() * 90000.0) as u32,
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
                debug!("Failed to send packet: {err}");
            }

            context.next_sequence_number = context.next_sequence_number.wrapping_add(1);
        }
    }
}

impl Drop for RtpSender {
    fn drop(&mut self) {
        self.should_close
            .store(true, std::sync::atomic::Ordering::Relaxed);
        match self.sender_thread.take() {
            Some(handle) => handle.join().unwrap(),
            None => error!("RTP sender thread was already joined."),
        }
    }
}

impl From<BindToPortError> for OutputInitError {
    fn from(value: BindToPortError) -> Self {
        match value {
            BindToPortError::SocketBind(err) => OutputInitError::SocketError(err),
            BindToPortError::PortAlreadyInUse(port) => OutputInitError::PortAlreadyInUse(port),
            BindToPortError::AllPortsAlreadyInUse {
                lower_bound,
                upper_bound,
            } => OutputInitError::AllPortsAlreadyInUse {
                lower_bound,
                upper_bound,
            },
        }
    }
}

struct TcpWritePacketStream {
    socket: socket2::Socket,
    should_close: Arc<AtomicBool>,
}

impl TcpWritePacketStream {
    fn new(socket: socket2::Socket, should_close: Arc<AtomicBool>) -> Self {
        socket
            .set_write_timeout(Some(Duration::from_millis(50)))
            .expect("Cannot set write timeout");
        Self {
            socket,
            should_close,
        }
    }

    fn write_packet(&mut self, data: bytes::Bytes) -> io::Result<()> {
        self.write_bytes(&u16::to_be_bytes(data.len() as u16))?;
        self.write_bytes(&data[..])?;
        io::Result::Ok(())
    }

    fn write_bytes(&mut self, data: &[u8]) -> io::Result<()> {
        let mut written_bytes = 0;
        loop {
            if written_bytes >= data.len() {
                return Ok(());
            }
            match self.socket.write(&data[written_bytes..]) {
                Ok(bytes) => {
                    written_bytes += bytes;
                }
                Err(err) => {
                    let should_close = self.should_close.load(std::sync::atomic::Ordering::Relaxed);
                    match err.kind() {
                        std::io::ErrorKind::WouldBlock if !should_close => {
                            continue;
                        }
                        _ => return io::Result::Err(err),
                    }
                }
            };
        }
    }
}
