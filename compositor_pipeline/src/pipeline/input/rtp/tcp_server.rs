use std::{
    io::{self, Read},
    net::TcpStream,
    sync::{atomic::AtomicBool, Arc},
    thread,
    time::Duration,
};

use bytes::BytesMut;
use compositor_render::error::ErrorStack;
use crossbeam_channel::Sender;
use log::info;

pub(super) fn run_tcp_server_receiver(
    socket: std::net::TcpListener,
    packets_tx: Sender<bytes::Bytes>,
    should_close: Arc<AtomicBool>,
) {
    let mut buffer = BytesMut::zeroed(65536);
    // make accept non blocking so we have a chance to handle should_close value
    socket
        .set_nonblocking(true)
        .expect("Cannot set non-blocking");

    loop {
        if should_close.load(std::sync::atomic::Ordering::Relaxed) {
            return;
        }

        // accept only one connection at the time
        let accept_result = socket.accept();
        let Ok((socket, _)) = accept_result else {
            thread::sleep(Duration::from_millis(50));
            continue;
        };
        info!("Connection accepted");

        socket
            .set_read_timeout(Some(Duration::from_millis(50)))
            .expect("Cannot set read timeout");

        loop {
            let mut len_bytes = [0u8; 2];
            if let Err(err) = (&socket).read_exact_with_should_close(&mut len_bytes, &should_close)
            {
                maybe_log_err(err);
                break;
            };
            let len = u16::from_be_bytes(len_bytes) as usize;

            if let Err(err) =
                (&socket).read_exact_with_should_close(&mut buffer[..len], &should_close)
            {
                maybe_log_err(err);
                break;
            };
            packets_tx
                .send(bytes::Bytes::copy_from_slice(&buffer[..len]))
                .unwrap();
        }
    }
}

fn maybe_log_err(err: io::Error) {
    if err.kind() != io::ErrorKind::WouldBlock {
        log::error!(
            "Unknown error when reading from TCP socket. {}",
            ErrorStack::new(&err).into_string()
        );
    }
}

trait TcpStreamExt {
    fn read_exact_with_should_close(
        &mut self,
        buf: &mut [u8],
        should_close: &Arc<AtomicBool>,
    ) -> io::Result<()>;
}

impl TcpStreamExt for &TcpStream {
    fn read_exact_with_should_close(
        &mut self,
        buf: &mut [u8],
        should_close: &Arc<AtomicBool>,
    ) -> io::Result<()> {
        loop {
            match self.read_exact(buf) {
                Ok(val) => return Ok(val),
                Err(err) => match err.kind() {
                    std::io::ErrorKind::WouldBlock
                        if should_close.load(std::sync::atomic::Ordering::Relaxed) =>
                    {
                        continue;
                    }
                    _ => return io::Result::Err(err),
                },
            };
        }
    }
}
