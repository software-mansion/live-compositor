use anyhow::{anyhow, Result};
use compositor_common::scene::Resolution;
use serde::{Deserialize, Serialize};
use std::{io::Cursor, net::SocketAddr, sync::Arc, thread};
use tiny_http::{Response, StatusCode};

use super::state::State;

#[derive(Serialize, Deserialize)]
struct RegisterInputRequest {
    pub port: u16,
}

#[derive(Serialize, Deserialize)]
struct RegisterOutputRequest {
    pub port: u16,
    pub resolution: Resolution,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Request {
    RegisterInput(RegisterInputRequest),
    RegisterOutput(RegisterOutputRequest),
    Init,
    Start,
}

#[allow(dead_code)]
pub struct Server {
    server: tiny_http::Server,
    state: Arc<State>,
}

impl Server {
    pub fn new(port: u16, state: Arc<State>) -> Self {
        Self {
            server: tiny_http::Server::http(SocketAddr::from(([0, 0, 0, 0], port))).unwrap(),
            state,
        }
    }

    pub fn start(self) {
        println!("Listening on port {}", self.server.server_addr());
        for mut raw_request in self.server.incoming_requests() {
            let result = self.handle_request_before_init(&mut raw_request);
            let should_abort = result.is_ok();
            Server::send_response(raw_request, result);
            if should_abort {
                break;
            }
        }
        thread::spawn(move || {
            for mut raw_request in self.server.incoming_requests() {
                let result = self.handle_request_after_init(&mut raw_request);
                Server::send_response(raw_request, result);
            }
        });
    }

    fn handle_request_after_init(&self, raw_request: &mut tiny_http::Request) -> Result<()> {
        let request = Server::parse_request(raw_request)?;
        match request {
            Request::Init => Err(anyhow!("Video composer is already configured.")),
            Request::RegisterInput(RegisterInputRequest { port }) => {
                self.state.register_input(port)
            }
            Request::RegisterOutput(RegisterOutputRequest { port, resolution }) => {
                self.state.register_output(port, resolution)
            }
            Request::Start => todo!(),
        }
    }

    fn handle_request_before_init(&self, raw_request: &mut tiny_http::Request) -> Result<()> {
        let request = Server::parse_request(raw_request)?;
        match request {
            Request::Init => {
                eprintln!("run any init code here");
                Ok(())
            }
            _ => Err(anyhow!(
                "No requests are supported before init request is completed."
            )),
        }
    }

    fn send_response(raw_request: tiny_http::Request, response: Result<()>) {
        let response_result = match response {
            Ok(_) => raw_request.respond(Response::new(
                StatusCode(200),
                vec![],
                Cursor::new("{}".as_bytes().to_vec()),
                None,
                None,
            )),
            Err(err) => raw_request.respond(Response::new(
                StatusCode(500),
                vec![],
                Cursor::new(format!("{}", err).into_bytes()),
                None,
                None,
            )),
        };
        if let Err(err) = response_result {
            eprintln!("Failed to send response {}.", err);
        }
    }

    fn parse_request(request: &mut tiny_http::Request) -> Result<Request> {
        Ok(serde_json::from_reader::<_, Request>(request.as_reader())?)
    }
}
