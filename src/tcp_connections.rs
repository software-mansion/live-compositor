use anyhow::Result;
use std::{
    io::{Read, Write},
    net::{Shutdown, TcpListener, TcpStream},
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc,
    },
    thread,
};

use crate::{pipeline::Pipeline, state::PendingConnection};

use super::state::State;

fn handle_connection(stream: TcpStream, state: Arc<State>) -> Result<()> {
    let local_address = stream.local_addr()?;
    if !local_address.ip().is_loopback() {
        stream.shutdown(Shutdown::Both)?;
        return Err(anyhow::anyhow!("Only local connections are supported."));
    }
    let port = local_address.port();
    state.pending_connections.insert(
        port,
        PendingConnection {
            port,
            tcp_stream: stream,
        },
    );
    Ok(())
}

pub fn listen_on_input(mut stream: TcpStream, pipeline: Arc<Pipeline>, input_id: u32) {
    thread::spawn(move || {
        let mut packet = vec![0; 65535];
        loop {
            let result: Result<()> = stream
                .read(packet.as_mut_slice())
                .map_err(|err| err.into())
                .and_then(|size| {
                    pipeline.push_input_data(
                        input_id,
                        bytes::Bytes::copy_from_slice(&packet[0..size]),
                    )?;
                    Ok(())
                });
            if let Err(err) = result {
                eprintln!("{}", err);
            }
        }
    });
}

pub fn listen_on_output(mut stream: TcpStream) -> Sender<bytes::Bytes> {
    let (tx, rx): (Sender<bytes::Bytes>, Receiver<bytes::Bytes>) = mpsc::channel();
    thread::spawn(move || loop {
        let data = match rx.recv() {
            Ok(data) => data,
            Err(_) => return,
        };
        if let Err(err) = stream.write_all(&data) {
            eprintln!("{}", err)
        }
    });
    tx
}

pub fn listen_for_new_connections(state: Arc<State>) -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8000")?;

    thread::spawn(move || {
        for stream in listener.incoming() {
            let state = state.clone();
            let result = stream
                .map_err(|err| err.into())
                .and_then(|s| handle_connection(s, state));
            if let Err(err) = result {
                eprintln!("{}", err)
            }
        }
    });
    Ok(())
}
