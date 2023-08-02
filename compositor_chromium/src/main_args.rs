use std::ffi::CString;

pub struct MainArgs {
    inner: chromium_sys::cef_main_args_t,
    _argv: Vec<*mut i8>,
}

impl MainArgs {
    pub fn from_env() -> Self {
        let mut argv: Vec<_> = std::env::args()
            .map(|arg| CString::new(arg).unwrap().into_raw())
            .collect();
        let inner = chromium_sys::cef_main_args_t {
            argc: argv.len() as i32,
            argv: argv.as_mut_ptr(),
        };

        Self { inner, _argv: argv }
    }

    pub fn raw_mut(&mut self) -> &mut chromium_sys::cef_main_args_t {
        &mut self.inner
    }
}
