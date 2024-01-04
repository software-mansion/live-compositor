use std::sync::Arc;

pub struct EventLoop {
    message_loop: Option<Arc<dyn MessageLoop>>,
}

impl EventLoop {
    pub fn new(message_loop: Option<Arc<dyn MessageLoop>>) -> Self {
        Self { message_loop }
    }

    /// Runs the event loop. It must run on the main thread.
    /// `fallback` is used when web rendering is disabled.
    /// Blocks the thread indefinitely.
    pub fn run_with_fallback(&self, fallback: impl FnOnce()) -> Result<(), EventLoopRunError> {
        match &self.message_loop {
            Some(message_loop) => message_loop.run()?,
            None => fallback(),
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EventLoopRunError {
    #[error("Event loop must run on the main thread")]
    WrongThread,
}

pub trait MessageLoop {
    fn run(&self) -> Result<(), EventLoopRunError>;
}
