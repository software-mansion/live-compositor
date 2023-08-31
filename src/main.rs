use compositor_chromium::cef;
use log::info;

mod api;
mod error;
mod http;
mod rtp_receiver;
mod rtp_sender;

fn main() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let target_path = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned();
    if cef::bundle_app(&target_path).is_err() {
        panic!("Build process helper first: cargo build --bin process_helper");
    }

    ffmpeg_next::format::network::init();

    http::Server::new(8001).run();

    info!("Received exit signal. Terminating...")
    // TODO: add graceful shutdown
}
