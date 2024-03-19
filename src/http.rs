use compositor_render::error::ErrorStack;
use log::{error, info};

use serde_json::json;
use signal_hook::{consts, iterator::Signals};
use std::{
    io::{Cursor, ErrorKind},
    net::SocketAddr,
    sync::Arc,
    thread,
};
use tiny_http::{Header, Response, StatusCode};

use crate::{
    api::{self, Api, ResponseHandler},
    config::config,
    error::ApiError,
    routes,
};

pub struct Server {
    server: tiny_http::Server,
    content_type_json: Header,
}

impl Server {
    pub fn new(port: u16) -> Arc<Self> {
        info!("Starting LiveCompositor with config:\n{:#?}", config());
        match tiny_http::Server::http(SocketAddr::from(([0, 0, 0, 0], port))) {
            Ok(server) => Self {
                server,
                content_type_json: Header::from_bytes(
                    &b"Content-Type"[..],
                    &b"application/json"[..],
                )
                .unwrap(),
            }
            .into(),
            Err(err) => {
                match err.downcast_ref::<std::io::Error>() {
                    Some(io_error) if io_error.kind() == ErrorKind::AddrInUse => {
                        error!("Port {port} is already used. Stop using it or specify port using LIVE_COMPOSITOR_API_PORT environment variable.");
                    }
                    Some(_) | None => {}
                };
                panic!("Failed to start video compositor HTTP server.\nError: {err}")
            }
        }
    }

    pub fn run(self: Arc<Self>) {
        info!("Listening on port {}", self.server.server_addr());
        let (mut api, event_loop) = Api::new().unwrap_or_else(|err| {
            panic!(
                "Failed to start event loop.\n{}",
                ErrorStack::new(&err).into_string()
            )
        });
        thread::spawn(move || {
            for raw_request in self.server.incoming_requests() {
                self.handle_request(&mut api, raw_request)
            }
        });

        let event_loop_fallback = || {
            let mut signals = Signals::new([consts::SIGINT]).unwrap();
            signals.forever().next();
        };
        if let Err(err) = event_loop.run_with_fallback(&event_loop_fallback) {
            panic!(
                "Failed to start event loop.\n{}",
                ErrorStack::new(&err).into_string()
            )
        }
    }

    fn handle_request(self: &Arc<Self>, api: &mut Api, mut raw_request: tiny_http::Request) {
        let response = routes::handle_request(api, &mut raw_request);
        match response {
            Ok(ResponseHandler::Ok) => {
                self.send_response(raw_request, api::Response::Ok {});
            }
            Ok(ResponseHandler::Response(response)) => {
                self.send_response(raw_request, response);
            }
            Err(err) => {
                self.send_err_response(raw_request, err);
            }
        }
    }

    fn send_response(&self, raw_request: tiny_http::Request, response: api::Response) {
        let response_result = serde_json::to_string(&response)
            .map_err(Into::into)
            .and_then(|body| {
                raw_request.respond(Response::new(
                    StatusCode(200),
                    vec![self.content_type_json.clone()],
                    Cursor::new(&body),
                    Some(body.len()),
                    None,
                ))
            });
        if let Err(err) = response_result {
            error!("Failed to send response {}.", err);
        }
    }

    fn send_err_response(&self, raw_request: tiny_http::Request, err: ApiError) {
        let response_result = serde_json::to_string(&json!({
            "msg": err.message,
            "stack": err.stack,
            "error_code": err.error_code,
        }))
        .map_err(Into::into)
        .and_then(|body| {
            raw_request.respond(Response::new(
                err.http_status_code,
                vec![self.content_type_json.clone()],
                Cursor::new(&body),
                Some(body.len()),
                None,
            ))
        });
        if let Err(err) = response_result {
            error!("Failed to send response {}.", err);
        }
    }
}
