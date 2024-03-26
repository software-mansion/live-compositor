use log::info;
use video_compositor::{logger, server};

fn main() {
    ffmpeg_next::format::network::init();
    logger::init_logger();

    server::run();

    info!("Received exit signal. Terminating...")
    // TODO: add graceful shutdown
}
