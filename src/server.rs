use compositor_render::error::ErrorStack;
use log::info;
use signal_hook::{consts, iterator::Signals};
use tracing::error;

use std::{net::SocketAddr, process, thread};
use tokio::runtime::Runtime;

use crate::{api::Api, config::config, routes::routes};

pub fn run() {
    run_on_port(config().api_port)
}

pub fn run_on_port(port: u16) {
    info!("Starting LiveCompositor with config:\n{:#?}", config());
    let (api, event_loop) = Api::new().unwrap_or_else(|err| {
        panic!(
            "Failed to start event loop.\n{}",
            ErrorStack::new(&err).into_string()
        )
    });

    thread::Builder::new()
        .name("HTTP server startup thread".to_string())
        .spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async { run_tokio_runtime(api, port).await });
        })
        .unwrap();

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

async fn run_tokio_runtime(api: Api, port: u16) {
    let app = routes(api);
    let listener = tokio::net::TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], port)))
        .await
        .unwrap();
    if let Err(err) = axum::serve(listener, app).await {
        error!(%err);
        process::exit(1)
    }
}
