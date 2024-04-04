use compositor_render::error::ErrorStack;
use log::info;
use signal_hook::{consts, iterator::Signals};
use tracing::error;

use std::{net::SocketAddr, process, thread};
use tokio::runtime::Runtime;

use crate::{
    config::{read_config, Config},
    logger::init_logger,
    routes::routes,
    state::ApiState,
};

pub fn run() {
    let config = read_config();
    init_logger(config.logger.clone());
    run_with_config(config)
}

pub fn run_with_config(config: Config) {
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
            let rt = Runtime::new().unwrap();
            rt.block_on(async { run_tokio_runtime(state).await });
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

async fn run_tokio_runtime(api: ApiState) {
    let port = api.config.api_port;
    let app = routes(api);
    let listener = tokio::net::TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], port)))
        .await
        .unwrap();
    if let Err(err) = axum::serve(listener, app).await {
        error!(%err);
        process::exit(1)
    }
}
