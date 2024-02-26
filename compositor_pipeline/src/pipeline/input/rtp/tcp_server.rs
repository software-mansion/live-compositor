use std::{
    collections::VecDeque,
    io::{self, Read},
    net::TcpStream,
    sync::{atomic::AtomicBool, Arc},
    thread,
    time::Duration,
};

use bytes::BytesMut;
use compositor_render::error::ErrorStack;
use crossbeam_channel::Sender;
use log::error;
use tracing::trace;

pub(super) fn run_tcp_server_receiver(
    socket: std::net::TcpListener,
    packets_tx: Sender<bytes::Bytes>,
    should_close: Arc<AtomicBool>,
) {
    // make accept non blocking so we have a chance to handle should_close value
    socket
        .set_nonblocking(true)
        .expect("Cannot set non-blocking");

    loop {
        if should_close.load(std::sync::atomic::Ordering::Relaxed) {
            return;
        }

        // accept only one connection at the time
        let Ok((socket, _)) = socket.accept() else {
            thread::sleep(Duration::from_millis(50));
            continue;
        };

        let mut socket = TcpReadPacketStream::new(socket, should_close.clone());
        loop {
            match socket.read_packet() {
                Ok(packet) => {
                    trace!(size_bytes = packet.len(), "Received RTP packet");
                    packets_tx.send(packet).unwrap();
                }
                Err(err) => {
                    error!(
                        "Error while reading from TCP socket: {}",
                        ErrorStack::new(&err).into_string()
                    );
                    break;
                }
            }
        }
    }
}

struct TcpReadPacketStream {
    socket: TcpStream,
    buf: VecDeque<u8>,
    read_buf: Vec<u8>,
    should_close: Arc<AtomicBool>,
}

impl TcpReadPacketStream {
    fn new(socket: TcpStream, should_close: Arc<AtomicBool>) -> Self {
        socket
            .set_read_timeout(Some(Duration::from_millis(50)))
            .expect("Cannot set read timeout");
        Self {
            socket,
            buf: VecDeque::new(),
            read_buf: vec![0; 65536],
            should_close,
        }
    }
    fn read_packet(&mut self) -> io::Result<bytes::Bytes> {
        self.read_until_buffer_size(2)?;

        let mut len_bytes = [0u8; 2];
        self.buf.read_exact(&mut len_bytes)?;
        let len = u16::from_be_bytes(len_bytes) as usize;

        self.read_until_buffer_size(len)?;
        let mut packet = BytesMut::zeroed(len);
        self.buf.read_exact(&mut packet[..])?;
        Ok(packet.freeze())
    }

    fn read_until_buffer_size(&mut self, buf_size: usize) -> io::Result<()> {
        loop {
            if self.buf.len() >= buf_size {
                return Ok(());
            }
            match self.socket.read(&mut self.read_buf) {
                Ok(read_bytes) => {
                    self.buf.extend(self.read_buf[0..read_bytes].iter());
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
