pub trait EventLoop {
    /// Runs the event loop. It must run on the main thread.
    /// `fallback` is used when web rendering is disabled.
    /// Blocks the thread indefinitely.
    fn run_with_fallback(&self, fallback: &dyn Fn()) -> Result<(), EventLoopRunError>;
}

#[derive(Debug, thiserror::Error)]
pub enum EventLoopRunError {
    #[error("Event loop must run on the main thread")]
    WrongThread,
}
