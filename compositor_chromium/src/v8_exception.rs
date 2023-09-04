use std::fmt::Display;

use crate::cef_string::CefString;

#[derive(Debug)]
pub struct V8Exception {
    exception: *mut chromium_sys::cef_v8exception_t,
}

impl Display for V8Exception {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "JavaScript Exception at [{},{}]: {}",
            self.line_number(),
            self.column_number(),
            self.message()
        )
    }
}

impl std::error::Error for V8Exception {}

impl V8Exception {
    pub(crate) fn new(exception: *mut chromium_sys::cef_v8exception_t) -> Self {
        Self { exception }
    }

    pub fn message(&self) -> String {
        unsafe {
            let get_message = (*self.exception).get_message.unwrap();
            let msg = get_message(self.exception);
            CefString::from_raw(msg)
        }
    }

    pub fn source_line(&self) -> String {
        unsafe {
            let get_source_line = (*self.exception).get_source_line.unwrap();
            let source_line = get_source_line(self.exception);
            CefString::from_raw(source_line)
        }
    }

    pub fn script_resource_name(&self) -> String {
        unsafe {
            let get_script_resource_name = (*self.exception).get_script_resource_name.unwrap();
            let script_resource_name = get_script_resource_name(self.exception);
            CefString::from_raw(script_resource_name)
        }
    }

    pub fn line_number(&self) -> i32 {
        unsafe {
            let get_line_number = (*self.exception).get_line_number.unwrap();
            get_line_number(self.exception)
        }
    }

    pub fn column_number(&self) -> i32 {
        unsafe {
            let get_column_number = (*self.exception).get_start_column.unwrap();
            get_column_number(self.exception)
        }
    }
}
