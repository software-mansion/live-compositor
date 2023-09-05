use std::error::Error;

use app::App;
use compositor_chromium::cef;

mod app;
mod handler;
mod state;

// Subprocess used by chromium
fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let app = App::new();
    let context = cef::Context::new_helper()?;
    let exit_code = context.execute_process(app);
    std::process::exit(exit_code);
}
