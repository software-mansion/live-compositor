use std::sync::{atomic::AtomicBool, Arc};

use bytes::{Bytes, BytesMut};
use crossbeam_channel::Sender;

pub(super) fn run_udp_receiver(
    socket: std::net::UdpSocket,
    packets_tx: Sender<bytes::Bytes>,
    should_close: Arc<AtomicBool>,
) {
    let mut buffer = BytesMut::zeroed(65536);

    loop {
        if should_close.load(std::sync::atomic::Ordering::Relaxed) {
            return;
        }

        // This can be faster if we batched sending the packets through the channel
        let (received_bytes, _) = match socket.recv_from(&mut buffer) {
            Ok(n) => n,
            Err(e) => match e.kind() {
                std::io::ErrorKind::WouldBlock => continue,
                _ => {
                    log::error!("Error while receiving UDP packet: {}", e);
                    continue;
                }
            },
        };

        packets_tx
            .send(Bytes::copy_from_slice(&buffer[..received_bytes]))
            .unwrap();
    }
}
