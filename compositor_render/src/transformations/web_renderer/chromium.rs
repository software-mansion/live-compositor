use std::sync::Arc;

use compositor_chromium::cef;
use crossbeam_channel::RecvError;
use log::info;

use super::browser::BrowserClient;

pub struct ChromiumContext {
    context: Option<Arc<cef::Context>>,
}

impl ChromiumContext {
    pub fn new(
        init_context: bool,
        show_fps: bool,
        disable_gpu: bool,
    ) -> Result<Self, ChromiumContextError> {
        if !init_context {
            info!("Init chromium context disabled");
            return Ok(Self { context: None });
        }

        info!("Init chromium context");

        let app = ChromiumApp {
            show_fps,
            disable_gpu,
        };
        let settings = cef::Settings {
            windowless_rendering_enabled: true,
            log_severity: cef::LogSeverity::Info,
            ..Default::default()
        };

        let context = Arc::new(cef::Context::new(app, settings)?);
        Ok(Self {
            context: Some(context),
        })
    }

    pub(super) fn start_browser(
        &self,
        url: &str,
        state: BrowserClient,
        frame_rate: i32,
    ) -> Result<cef::Browser, ChromiumContextError> {
        let context = self
            .context
            .as_ref()
            .ok_or(ChromiumContextError::NoContext)?;

        let window_info = cef::WindowInfo {
            windowless_rendering_enabled: true,
        };
        let settings = cef::BrowserSettings {
            windowless_frame_rate: frame_rate,
        };

        let (tx, rx) = crossbeam_channel::bounded(1);
        let task = cef::Task::new(move || {
            let result = context.start_browser(state, window_info, settings, url);
            tx.send(result).unwrap();
        });

        task.run(cef::ThreadId::UI);
        Ok(rx.recv()??)
    }

    /// Runs chromium's message loop. This call is blocking. It has to be used on the main thread
    pub fn event_loop(&self) -> Option<EventLoop> {
        self.context.clone().map(EventLoop::new)
    }
}

struct ChromiumApp {
    show_fps: bool,
    disable_gpu: bool,
}

impl cef::App for ChromiumApp {
    type RenderProcessHandlerType = ();

    fn on_before_command_line_processing(
        &mut self,
        process_type: String,
        command_line: &mut cef::CommandLine,
    ) {
        // Execute only on the main process
        if !process_type.is_empty() {
            return;
        }

        // OSR will not work without this on MacOS
        #[cfg(target_os = "macos")]
        command_line.append_switch("use-mock-keychain");

        if self.show_fps {
            command_line.append_switch("show-fps-counter")
        }
        if self.disable_gpu {
            command_line.append_switch("disable-gpu");
        }

        command_line.append_switch("disable-gpu-shader-disk-cache");
        command_line.append_switch_with_value("autoplay-policy", "no-user-gesture-required");
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ChromiumContextError {
    #[error("Chromium context failed: {0}")]
    ContextFailure(#[from] cef::ContextError),

    #[error("Thread communication failed")]
    ThreadNoResponse(#[from] RecvError),

    #[error("Chromium context not initialized")]
    NoContext,

    #[error("Chromium message loop can only run on the main thread")]
    WrongThreadForMessageLoop,
}

pub struct EventLoop {
    chromium_context: Arc<cef::Context>,
}

impl EventLoop {
    pub fn new(chromium_context: Arc<cef::Context>) -> Self {
        Self { chromium_context }
    }

    /// Runs chrome's message loop. Must be run on the main thread
    pub fn run(&self) -> Result<(), EventLoopRunError> {
        if !self.chromium_context.currently_on_thread(cef::ThreadId::UI) {
            return Err(EventLoopRunError::WrongThread);
        }

        self.chromium_context.run_message_loop();
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EventLoopRunError {
    #[error("Event loop must run on the main thread")]
    WrongThread,
}
