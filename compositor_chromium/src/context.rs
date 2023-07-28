use std::{ffi::CString, io, path::PathBuf};

use crate::{
    cef::*, cef_ref::CefRefPtr, cef_string::CefString, main_args::MainArgs, post_task::Task,
};

pub struct Context {
    _priv: (),
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            chromium_sys::cef_shutdown();
            chromium_sys::cef_unload_library();
        }
    }
}

impl Context {
    #[cfg(target_os = "macos")]
    pub fn new<A: App>(app: A, settings: Settings) -> Result<Self, ContextError> {
        let framework_path = PathBuf::from(std::env::current_exe()?)
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("Frameworks")
            .join("Chromium Embedded Framework.framework")
            .join("Chromium Embedded Framework");
        let framework_path = CString::new(framework_path.display().to_string()).unwrap();

        let is_loaded = unsafe { chromium_sys::cef_load_library(framework_path.as_ptr()) };
        if is_loaded != 1 {
            return Err(ContextError::FrameworkNotLoaded);
        }

        Self::init(app, settings)?;
        Ok(Context { _priv: () })
    }

    fn init<A: App>(app: A, settings: Settings) -> Result<(), ContextError> {
        let mut main_args = MainArgs::from_env();
        let settings = settings.into_raw();
        let mut app = CefRefPtr::new(AppWrapper(app));

        let init_result = unsafe {
            chromium_sys::cef_initialize(main_args.raw_mut(), &settings, app, std::ptr::null_mut())
        };

        if init_result != 1 {
            return Err(ContextError::FrameworkInitFailed);
        }

        Ok(())
    }

    pub fn start_browser<'a, C: Client>(
        &'a self,
        client: C,
        window_info: WindowInfo,
        settings: BrowserSettings,
        url: String,
    ) {
        let task = Task::new(move || unsafe {
            let client = CefRefPtr::new(ClientWrapper(client));
            let window_info = window_info.into_raw();
            let settings = settings.into_raw();
            let url = CefString::new_raw(url);
            chromium_sys::cef_browser_host_create_browser_sync(
                &window_info,
                client,
                &url,
                &settings,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
        });

        task.run(chromium_sys::cef_thread_id_t_TID_UI);
    }

    pub fn run_message_loop(&self) {
        unsafe {
            chromium_sys::cef_run_message_loop();
        }
    }

    pub fn do_message_loop_work(&self) {
        unsafe {
            // TODO: The use of this function is not recommended.
            // We should use multithreaded message loop which is unfortunately not supported on MacOS
            chromium_sys::cef_do_message_loop_work();
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ContextError {
    #[error("IO operation failed")]
    IOError(#[from] io::Error),

    #[error("Failed to load chromium framework lib")]
    FrameworkNotLoaded,

    #[error("Failed to init chromium framework")]
    FrameworkInitFailed,

    #[error("Failed to start browser session")]
    StartBrowserFailed,
}
