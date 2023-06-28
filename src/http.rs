use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    io::{Cursor, Read},
    sync::Arc,
    thread,
};
use tiny_http::{Response, Server, StatusCode};

use super::state::State;

#[derive(Serialize, Deserialize)]
struct RegisterInputRequest {
    pub port: u16,
    pub input_id: u32,
}

#[derive(Serialize, Deserialize)]
struct RegisterOutputRequest {
    pub port: u16,
    pub output_id: u32,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Request {
    RegisterInput(RegisterInputRequest),
    RegisterOutput(RegisterOutputRequest),
}

fn handle_event(state: Arc<State>, event: Request) -> Result<()> {
    match event {
        Request::RegisterInput(RegisterInputRequest { port, input_id }) => {
            state.add_input(port, input_id)?;
            Ok(())
        }
        Request::RegisterOutput(RegisterOutputRequest { port, output_id }) => {
            state.add_output(port, output_id)?;
            Ok(())
        }
    }
}

fn handle_request(state: Arc<State>, request: &mut tiny_http::Request) -> Response<impl Read> {
    let event = match serde_json::from_reader::<_, Request>(request.as_reader()) {
        Ok(event) => event,
        Err(err) => {
            return Response::new(
                StatusCode(500),
                vec![],
                Cursor::new(format!("{}", err).into_bytes()),
                None,
                None,
            );
        }
    };
    if let Err(err) = handle_event(state, event) {
        return Response::new(
            StatusCode(500),
            vec![],
            Cursor::new(format!("{}", err).into_bytes()),
            None,
            None,
        );
    }
    Response::new(
        StatusCode(200),
        vec![],
        Cursor::new("{}".as_bytes().to_vec()),
        None,
        None,
    )
}

pub fn listen_for_events(state: Arc<State>) {
    let server = Server::http("0.0.0.0:8001").unwrap();

    thread::spawn(move || {
        for mut request in server.incoming_requests() {
            let response = handle_request(state.clone(), &mut request);
            if let Err(err) = request.respond(response) {
                eprintln!("{}", err)
            }
        }
    });
}
