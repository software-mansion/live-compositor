use std::env;

use compositor_chromium::cef::bundle_for_development;
use log::info;

use crate::http::API_PORT_ENV;

mod api;
mod error;
mod http;
mod rtp_receiver;
mod rtp_sender;
mod types;

#[cfg(test)]
mod snapshot_tests;

fn main() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let target_path = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned();
    if bundle_for_development(&target_path).is_err() {
        panic!("Build process helper first. For release profile use: cargo build -r --bin process_helper");
    }

    ffmpeg_next::format::network::init();

    let port = env::var(API_PORT_ENV).unwrap_or_else(|_| "8001".to_string());
    http::Server::new(port.parse::<u16>().unwrap()).run();

    info!("Received exit signal. Terminating...")
    // TODO: add graceful shutdown
}
