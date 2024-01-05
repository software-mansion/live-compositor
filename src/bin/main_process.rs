use log::info;
use video_compositor::{
    config::config,
    http::{self},
    logger,
};

fn main() {
    ffmpeg_next::format::network::init();
    logger::init_logger();

    http::Server::new(config().api_port).run();

    info!("Received exit signal. Terminating...")
    // TODO: add graceful shutdown
}
