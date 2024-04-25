use live_compositor::server;
use log::info;

fn main() {
    ffmpeg_next::format::network::init();

    server::run();

    info!("Received exit signal. Terminating...")
    // TODO: add graceful shutdown
}
