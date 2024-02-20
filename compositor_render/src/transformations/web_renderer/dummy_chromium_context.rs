use std::sync::Arc;

use crate::{
    event_loop::{EventLoop, EventLoopRunError},
    types::Framerate,
};

use super::WebRendererInitOptions;

pub struct ChromiumContext;

impl ChromiumContext {
    pub(crate) fn new(
        opts: WebRendererInitOptions,
        _framerate: Framerate,
    ) -> Result<Self, WebRendererContextError> {
        if opts.enable {
            return Err(WebRendererContextError::WebRenderingNotAvailable);
        }

        Ok(Self)
    }

    pub fn event_loop(&self) -> Arc<dyn EventLoop> {
        Arc::new(FallbackEventLoop)
    }
}

struct FallbackEventLoop;

impl EventLoop for FallbackEventLoop {
    fn run_with_fallback(&self, fallback: &dyn Fn()) -> Result<(), EventLoopRunError> {
        fallback();
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WebRendererContextError {
    #[error("Web rendering feature is not available")]
    WebRenderingNotAvailable,
}
