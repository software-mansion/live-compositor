use std::sync::Arc;

use compositor_chromium::cef;
use compositor_common::Framerate;
use crossbeam_channel::RecvError;
use log::info;

use crate::WebRendererOptions;

use super::browser::BrowserClient;

pub struct ChromiumContext {
    context: Option<Arc<cef::Context>>,
    framerate: Framerate,
}

impl ChromiumContext {
    pub(crate) fn new(
        opts: WebRendererOptions,
        framerate: Framerate,
    ) -> Result<Self, WebRendererContextError> {
        if !opts.init {
            info!("Chromium context disabled");
            return Ok(Self {
                framerate,
                context: None,
            });
        }

        info!("Init chromium context");

        let app = ChromiumApp {
            show_fps: false,
            disable_gpu: opts.disable_gpu,
        };
        let settings = cef::Settings {
            windowless_rendering_enabled: true,
            log_severity: cef::LogSeverity::Info,
            ..Default::default()
        };

        let context = Arc::new(cef::Context::new(app, settings)?);
        Ok(Self {
            framerate,
            context: Some(context),
        })
    }

    pub(super) fn start_browser(
        &self,
        url: &str,
        state: BrowserClient,
    ) -> Result<cef::Browser, WebRendererContextError> {
        let context = self
            .context
            .as_ref()
            .ok_or(WebRendererContextError::NoContext)?;

        let window_info = cef::WindowInfo {
            windowless_rendering_enabled: true,
        };
        let settings = cef::BrowserSettings {
            windowless_frame_rate: self.framerate.0 as i32,
        };

        let (tx, rx) = crossbeam_channel::bounded(1);
        let task = cef::Task::new(move || {
            let result = context.start_browser(state, window_info, settings, url);
            tx.send(result).unwrap();
        });

        task.run(cef::ThreadId::UI);
        Ok(rx.recv()??)
    }

    pub fn cef_context(&self) -> Option<Arc<cef::Context>> {
        self.context.clone()
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
pub enum WebRendererContextError {
    #[error("Chromium context failed: {0}")]
    ContextFailure(#[from] cef::ContextError),

    #[error("Thread communication failed")]
    ThreadNoResponse(#[from] RecvError),

    #[error("Chromium context not initialized")]
    NoContext,

    #[error("Chromium message loop can only run on the main thread")]
    WrongThreadForMessageLoop,
}
