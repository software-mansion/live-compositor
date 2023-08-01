use compositor_chromium::cef;
use log::info;
use std::{error::Error, process::ExitCode};

struct App;

impl cef::App for App {
    type RenderProcessHandlerType = RenderProcessHandler;

    fn get_render_process_handler(&self) -> Option<Self::RenderProcessHandlerType> {
        Some(RenderProcessHandler)
    }
}

struct RenderProcessHandler;

impl cef::RenderProcessHandler for RenderProcessHandler {
    fn on_process_message_received(
        &mut self,
        browser: cef::Browser<'_>,
        frame: cef::Frame<'_>,
        source_process: cef::ProcessId,
        message: cef::ProcessMessage,
    ) -> bool {
        // TODO: Implement this
        info!("Message received: {}", message.get_name());
        let bytes = message.read_bytes(0);
        dbg!(bytes);
        false
    }
}

// Subprocess used by chromium
fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let subprocess_type = std::env::args().find_map(|a| a.strip_prefix("--type=").map(ToOwned::to_owned)).unwrap();

    info!("Chromium {subprocess_type} subprocess started");
    let context = cef::Context::new_helper()?;
    let exit_code = context.execute_process(App);
    std::process::exit(exit_code);
}
