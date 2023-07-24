use std::ffi::CString;

pub struct MainArgs {
    argv: Vec<*mut i8>,
    inner: sys::cef_main_args_t,
}

impl MainArgs {
    pub fn from_env() -> Self {
        let mut argv: Vec<_> = std::env::args()
            .map(|arg| CString::new(arg).unwrap().into_raw())
            .collect();
        let inner = sys::cef_main_args_t {
            argc: argv.len() as i32,
            argv: argv.as_mut_ptr(),
        };

        Self { argv, inner }
    }

    pub fn raw_mut(&mut self) -> &mut sys::cef_main_args_t {
        &mut self.inner
    }
}
