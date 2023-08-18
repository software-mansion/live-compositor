use std::sync::Arc;

use compositor_chromium::cef;

pub struct EventLoop {
    cef_ctx: Option<Arc<cef::Context>>,
}

impl EventLoop {
    pub fn new(cef_ctx: Option<Arc<cef::Context>>) -> Self {
        Self { cef_ctx }
    }

    /// Runs the event loop. It must run on the main thread.
    /// `fallback` is used when web rendering is disabled.
    /// Blocks the thread indefinitely.
    pub fn run_with_fallback(&self, fallback: impl FnOnce()) -> Result<(), EventLoopRunError> {
        match &self.cef_ctx {
            Some(ctx) => self.cef_event_loop(ctx)?,
            None => fallback(),
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
}

#[derive(Debug, thiserror::Error)]
pub enum EventLoopRunError {
    #[error("Event loop must run on the main thread")]
    WrongThread,
}
