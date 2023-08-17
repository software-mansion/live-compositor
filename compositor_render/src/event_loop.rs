use std::sync::Arc;

use compositor_chromium::cef;
use signal_hook::{consts, iterator::Signals};

pub struct EventLoop {
    cef_ctx: Option<Arc<cef::Context>>,
}

impl EventLoop {
    pub fn new(cef_ctx: Option<Arc<cef::Context>>) -> Self {
        Self { cef_ctx }
    }

    /// Runs the event loop. It must run on the main thread.
    /// Blocks the thread indefinitely
    pub fn run(&self) -> Result<(), EventLoopRunError> {
        match &self.cef_ctx {
            Some(ctx) => self.cef_event_loop(ctx)?,
            None => self.fallback_event_loop(),
        }

        Ok(())
    }

    fn cef_event_loop(&self, ctx: &cef::Context) -> Result<(), EventLoopRunError> {
        if !ctx.currently_on_thread(cef::ThreadId::UI) {
            return Err(EventLoopRunError::WrongThread);
        }

        ctx.run_message_loop();
        Ok(())
    }

    /// Fallback event loop used when web renderer is disabled
    fn fallback_event_loop(&self) {
        let mut signals = Signals::new([consts::SIGINT]).unwrap();
        signals.forever().next();
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EventLoopRunError {
    #[error("Event loop must run on the main thread")]
    WrongThread,
}
