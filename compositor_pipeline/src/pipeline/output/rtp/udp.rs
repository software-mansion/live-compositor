use tracing::{debug, error, trace};

use crate::{error::OutputInitError, pipeline::Port};

use super::packet_stream::PacketStream;

pub(super) fn udp_socket(ip: &str, port: Port) -> Result<(socket2::Socket, Port), OutputInitError> {
    let socket = std::net::UdpSocket::bind(std::net::SocketAddrV4::new(
        std::net::Ipv4Addr::UNSPECIFIED,
        0,
    ))?;

    socket.connect((ip, port.0))?;
    Ok((socket.into(), port))
}

/// this assumes, that a "packet" contains data about a single frame (access unit)
pub(super) fn run_udp_sender_thread(socket: socket2::Socket, packet_stream: PacketStream) {
    for chunk in packet_stream {
        let chunk = match chunk {
            Ok(chunk) => chunk,
            Err(err) => {
                error!("Failed to payload a packet: {}", err);
                continue;
            }
        };
        trace!(size_bytes = chunk.len(), "Send RTP UDP packet.");
        if let Err(err) = socket.send(&chunk) {
            debug!("Failed to send packet: {err}");
        };
    }
}
