use log::info;

mod api;
mod http;
mod rtp_receiver;
mod rtp_sender;

fn main() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );
    ffmpeg_next::format::network::init();

    http::Server::new(8001).start();

    info!("Received exit signal. Terminating...")
    // TODO: add graceful shutdown
}
