use std::os::raw::c_void;

use crate::cef_string::CefString;

pub struct ProcessMessage {
    pub(crate) inner: *mut chromium_sys::cef_process_message_t,
}

impl ProcessMessage {
    pub fn new(name: &str) -> Self {
        let name = CefString::new_raw(name);
        let inner = unsafe { chromium_sys::cef_process_message_create(&name) };
        Self { inner }
    }

    pub fn write_binary(&mut self, data: &[u8]) {
        unsafe {
            let get_argument_list = (&mut *self.inner).get_argument_list.unwrap();
            let args = get_argument_list(self.inner);
            let set_binary = (&mut *args).set_binary.unwrap();
            let binary_value =
                chromium_sys::cef_binary_value_create(data.as_ptr() as *const c_void, data.len());

            set_binary(args, 0, binary_value);
        }
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum ProcessId {
    Browser = chromium_sys::cef_process_id_t_PID_BROWSER,
    Renderer = chromium_sys::cef_process_id_t_PID_RENDERER,
}
