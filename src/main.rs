use std::sync::Arc;

use signal_hook::{consts, iterator::Signals};
use state::Pipeline;

use crate::state::State;

mod http;
mod rtp_receiver;
mod rtp_sender;
mod state;

fn main() {
    ffmpeg_next::format::network::init();
    let pipeline = Arc::new(Pipeline::new());
    let state = Arc::new(State::new(pipeline));

    http::Server::new(8001, state).start();

    let mut signals = Signals::new([consts::SIGINT]).unwrap();
    signals.forever().next();
    eprintln!("Received exit signal. Terminating...")
    // TODO: add graceful shutdown
}
