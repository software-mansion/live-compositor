use std::io;

use crate::{cef::*, cef_ref::CefRefPtr, cef_string::CefString, main_args::MainArgs};

pub struct Context {
    _priv: (),
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            chromium_sys::cef_shutdown();

            #[cfg(target_os = "macos")]
            chromium_sys::cef_unload_library();
        }
    }
}

impl Context {
    pub fn new<A: App>(app: A, settings: Settings) -> Result<Self, ContextError> {
        #[cfg(target_os = "macos")]
        {
            let framework_path = std::env::current_exe()?
                .parent()
                .unwrap()
                .join("video_compositor.app")
                .join("Contents")
                .join("Frameworks")
                .join("Chromium Embedded Framework.framework")
                .join("Chromium Embedded Framework");

            Self::load_framework(framework_path)?;
        }

        let mut main_args = MainArgs::from_env();
        let settings = settings.into_raw();
        let app = CefRefPtr::new_ptr(AppWrapper(app));

        let init_result = unsafe {
            chromium_sys::cef_initialize(main_args.raw_mut(), &settings, app, std::ptr::null_mut())
        };

        if init_result != 1 {
            return Err(ContextError::FrameworkInitFailed);
        }

        Ok(Context { _priv: () })
    }

    pub fn new_helper() -> Result<Self, ContextError> {
        #[cfg(target_os = "macos")]
        {
            let framework_path = std::env::current_exe()?
                .parent()
                .unwrap()
                .join("..")
                .join("..")
                .join("..")
                .join("Chromium Embedded Framework.framework")
                .join("Chromium Embedded Framework");

            Self::load_framework(framework_path)?;
        }

        Ok(Context { _priv: () })
    }

    #[cfg(target_os = "macos")]
    fn load_framework(framework_path: std::path::PathBuf) -> Result<(), ContextError> {
        use std::ffi::CString;
        let framework_path = CString::new(framework_path.display().to_string()).unwrap();
        let is_loaded = unsafe { chromium_sys::cef_load_library(framework_path.as_ptr()) };
        if is_loaded != 1 {
            return Err(ContextError::FrameworkNotLoaded);
        }

        Ok(())
    }

    pub fn execute_process<A: App>(&self, app: A) -> i32 {
        let mut main_args = MainArgs::from_env();
        let app = CefRefPtr::new_ptr(AppWrapper(app));
        unsafe { chromium_sys::cef_execute_process(main_args.raw_mut(), app, std::ptr::null_mut()) }
    }

    pub fn start_browser<C: Client>(
        &self,
        client: C,
        window_info: WindowInfo,
        settings: BrowserSettings,
        url: String,
    ) -> Result<Browser<'_>, ContextError> {
        let client = CefRefPtr::new_ptr(ClientWrapper(client));
        let window_info = window_info.into_raw();
        let settings = settings.into_raw();
        let url = CefString::new_raw(url);
        let browser = unsafe {
            chromium_sys::cef_browser_host_create_browser_sync(
                &window_info,
                client,
                &url,
                &settings,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        if browser.is_null() {
            return Err(ContextError::StartBrowserFailed);
        }

        Ok(Browser::new(browser))
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
