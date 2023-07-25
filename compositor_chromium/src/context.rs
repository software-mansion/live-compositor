use std::{ffi::CString, io, path::PathBuf};

use crate::{
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
    pub fn new(settings: Settings) -> Result<Self, ContextError> {
        let framework_path = PathBuf::from(std::env::current_exe()?)
            .parent()
            .unwrap()
            .join("..")
            .join("Chromium Embedded Framework.framework")
            .join("Chromium Embedded Framework");

        let is_loaded = unsafe {
            let framework_path = CString::new(framework_path.display().to_string()).unwrap();
            chromium_sys::cef_load_library(framework_path.as_ptr())
        };

        if is_loaded != 1 {
            return Err(ContextError::FrameworkNotLoaded);
        }

        Self::init();
        Ok(Context { _priv: () })
    }

    fn init() -> Result<(), ContextError> {
        let mut main_args = MainArgs::from_env();
        let settings = SettingsBuilder::new().build();

        let init_result = unsafe {
            chromium_sys::cef_initialize(
                main_args.raw_mut(),
                settings.raw(),
                std::ptr::null_mut(),
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
