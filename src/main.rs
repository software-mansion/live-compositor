use log::info;
use signal_hook::{consts, iterator::Signals};

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

    let mut signals = Signals::new([consts::SIGINT]).unwrap();
    signals.forever().next();
    info!("Received exit signal. Terminating...")
    // TODO: add graceful shutdown
}
