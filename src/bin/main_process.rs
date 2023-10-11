use std::env;

use log::info;
use video_compositor::http;

pub const API_PORT_ENV: &str = "MEMBRANE_VIDEO_COMPOSITOR_API_PORT";

fn main() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    ffmpeg_next::format::network::init();

    let port = env::var(API_PORT_ENV).unwrap_or_else(|_| "8001".to_string());
    http::Server::new(port.parse::<u16>().unwrap()).run();

    info!("Received exit signal. Terminating...")
    // TODO: add graceful shutdown
}
