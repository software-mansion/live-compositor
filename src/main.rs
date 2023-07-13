use std::sync::Arc;

use pipeline::Pipeline;
use signal_hook::{consts::SIGINT, iterator::Signals};

use crate::state::State;

mod decoder;
mod encoder;
mod http;
mod pipeline;
mod queue;
mod rtp;
mod state;
mod tcp_connections;

fn main() {
    let pipeline = Arc::new(Pipeline::new());
    let state = Arc::new(State::new(pipeline.clone()));

    tcp_connections::listen_for_new_connections(state.clone()).unwrap();
    http::listen_for_events(state);

    pipeline.start();

    let mut signals = Signals::new([SIGINT]).unwrap();
    signals.forever();
}
