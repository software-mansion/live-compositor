use compositor_render::{error::ErrorStack, EventLoop};
use crossbeam_channel::{Receiver, Sender};
use log::info;
use signal_hook::{consts, iterator::Signals};
use tracing::error;

use std::{net::SocketAddr, process, sync::Arc, thread};
use tokio::runtime::Runtime;

use crate::{
    api::Api,
    config::{read_config, Config},
    logger::init_logger,
    routes::routes,
};

pub fn run() {
    let config = read_config();
    init_logger(config.logger.clone());
    run_with_config(config)
}

pub fn run_with_config(config: Config) {
    info!("Starting LiveCompositor with config:\n{:#?}", config);
    let (event_loop, should_close_sender) = start_api(config);

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

    should_close_sender.send(()).unwrap();
}

pub fn start_api(config: Config) -> (Arc<dyn EventLoop>, Sender<()>) {
    let (should_close_sender, should_close_receiver) = crossbeam_channel::bounded(1);
    let (api, event_loop) = Api::new(config).unwrap();
    thread::Builder::new()
        .name("HTTP server startup thread".to_string())
        .spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                run_tokio_runtime(api, should_close_receiver).await;
            });
        })
        .unwrap();

    (event_loop, should_close_sender)
}

async fn run_tokio_runtime(api: Api, should_close: Receiver<()>) {
    let port = api.config.api_port;
    let app = routes(api);
    let listener = tokio::net::TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], port)))
        .await
        .unwrap();

    if let Err(err) = axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            should_close.recv().unwrap();
        })
        .await
    {
        error!(%err);
        process::exit(1)
    }
}
