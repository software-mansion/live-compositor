use std::sync::Arc;

use compositor_common::Framerate;
use log::info;
use signal_hook::{consts, iterator::Signals};
use state::Pipeline;

use crate::state::State;

mod http;
mod rtp_receiver;
mod rtp_sender;
mod state;

fn main() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );
    ffmpeg_next::format::network::init();

    let pipeline = Arc::new(Pipeline::new(Framerate(30)));
    let state = Arc::new(State::new(pipeline));

    http::Server::new(8001, state).start();

    let mut signals = Signals::new([consts::SIGINT]).unwrap();
    signals.forever().next();
    info!("Received exit signal. Terminating...")
    // TODO: add graceful shutdown
}
