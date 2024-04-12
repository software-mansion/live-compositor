use compositor_render::error::ErrorStack;
use crossbeam_channel::Receiver;
use log::info;
use signal_hook::{consts, iterator::Signals};
use tracing::error;

use std::{net::SocketAddr, process, thread};
use tokio::runtime::Runtime;

use crate::{config::read_config, logger::init_logger, routes::routes, state::ApiState};

pub fn run() {
    let config = read_config();
    init_logger(config.logger.clone());

    info!("Starting LiveCompositor with config:\n{:#?}", config);
    let (state, event_loop) = ApiState::new(config).unwrap_or_else(|err| {
        panic!(
            "Failed to start event loop.\n{}",
            ErrorStack::new(&err).into_string()
        )
    });

    thread::Builder::new()
        .name("HTTP server startup thread".to_string())
        .spawn(move || {
            let (_should_close_sender, should_close_receiver) = crossbeam_channel::bounded(1);
            if let Err(err) = run_api(state, should_close_receiver) {
                error!(%err);
                process::exit(1);
            }
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

pub fn run_api(state: ApiState, should_close: Receiver<()>) -> tokio::io::Result<()> {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let port = state.config.api_port;
        let app = routes(state);
        let listener =
            tokio::net::TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], port))).await?;

        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                should_close.recv().unwrap();
            })
            .await
    })
}
