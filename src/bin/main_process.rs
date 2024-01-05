use std::env;

use log::info;
use video_compositor::{
    http::{self, API_PORT_ENV},
    logger,
};

fn main() {
    ffmpeg_next::format::network::init();
    logger::init_logger();

    let port = env::var(API_PORT_ENV).unwrap_or_else(|_| "8001".to_string());
    http::Server::new(port.parse::<u16>().unwrap()).run();

    info!("Received exit signal. Terminating...")
    // TODO: add graceful shutdown
}
