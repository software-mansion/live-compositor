use std::{ffi::CString, io, path::PathBuf};

use crate::{
    app::{App, AppWrapper},
    cef_ref::CefRefPtr,
    main_args::MainArgs,
    settings::{Settings, SettingsBuilder},
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
    pub fn new<T: App>(app: T, settings: Settings) -> Result<Self, ContextError> {
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

    fn init<T: App>(app: T, settings: Settings) -> Result<(), ContextError> {
        let mut main_args = MainArgs::from_env();
        let mut app = CefRefPtr::new(AppWrapper(app));

        let init_result = unsafe {
            chromium_sys::cef_initialize(
                main_args.raw_mut(),
                settings.raw(),
                app,
                std::ptr::null_mut(),
            )
        };

        if init_result != 1 {
            return Err(ContextError::FrameworkInitFailed);
        }

        Ok(())
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
}
