use compositor_chromium::cef;
use log::info;
use std::error::Error;

struct App;

impl cef::App for App {
    type RenderProcessHandlerType = RenderProcessHandler;

    fn on_before_command_line_processing(
        &mut self,
        process_type: String,
        _command_line: &mut cef::CommandLine,
    ) {
        info!("Chromium {process_type} subprocess started");
    }

    fn render_process_handler(&self) -> Option<Self::RenderProcessHandlerType> {
        Some(RenderProcessHandler)
    }
}

struct RenderProcessHandler;

impl cef::RenderProcessHandler for RenderProcessHandler {
    fn on_process_message_received(
        &mut self,
        _browser: &cef::Browser,
        _frame: &cef::Frame,
        _source_process: cef::ProcessId,
        message: &cef::ProcessMessage,
    ) -> bool {
        // TODO: Implement this
        info!("Message received: {}", message.name());
        let ctx = _frame.v8_context().unwrap();
        let result = ctx.eval("let a = 2+ 1; a");
        match result {
            Ok(value) => dbg!(value.),
        }
        false
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
