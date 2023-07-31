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
    fn on_context_created(&mut self, browser: cef::Browser<'_>) {
        info!("Context created");
        // TODO: Implement it
        std::fs::write("test.txt", "TEST");
    }
}

// Subprocess used by chromium
fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );
    let context = cef::Context::new_helper()?;
    let exit_code = context.execute_process(App);
    std::process::exit(exit_code);
}
