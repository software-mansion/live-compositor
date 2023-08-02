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

    pub fn get_name(&self) -> String {
        unsafe {
            let get_name = (*self.inner).get_name.unwrap();
            CefString::from_raw(get_name(self.inner))
        }
    }

    pub fn write_binary(&mut self, index: usize, data: &[u8]) {
        unsafe {
            let get_argument_list = (*self.inner).get_argument_list.unwrap();
            let args = get_argument_list(self.inner);
            let set_binary = (*args).set_binary.unwrap();
            let binary_value =
                chromium_sys::cef_binary_value_create(data.as_ptr() as *const c_void, data.len());

            set_binary(args, index, binary_value);
        }
    }

    // TODO: Rewrite
    pub fn read_bytes(&self, index: usize) -> Vec<u8> {
        let mut data = Vec::new();

        unsafe {
            let get_argument_list = (*self.inner).get_argument_list.unwrap();
            let args = get_argument_list(self.inner);
            let get_binary = (*args).get_binary.unwrap();
            let binary = get_binary(args, index);
            let get_data = (*binary).get_data.unwrap();
            let get_data_size = (*binary).get_size.unwrap();

            let data_size = get_data_size(binary);
            data.resize(data_size, 0);

            let mut read_bytes = 0;
            while read_bytes < data_size {
                let data_ptr = data.as_mut_ptr().add(read_bytes);
                read_bytes += get_data(binary, data_ptr as *mut c_void, data_size, read_bytes);
            }
        }

        data
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum ProcessId {
    Browser = chromium_sys::cef_process_id_t_PID_BROWSER,
    Renderer = chromium_sys::cef_process_id_t_PID_RENDERER,
}

impl From<chromium_sys::cef_process_id_t> for ProcessId {
    fn from(value: chromium_sys::cef_process_id_t) -> Self {
        match value {
            chromium_sys::cef_process_id_t_PID_BROWSER => Self::Browser,
            chromium_sys::cef_process_id_t_PID_RENDERER => Self::Renderer,
            _ => unreachable!(),
        }
    }
}
