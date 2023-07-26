use std::ops::Deref;

use widestring::U16CString;

pub struct CefString;

impl CefString {
    pub fn new_raw<S: Into<String>>(s: S) -> chromium_sys::cef_string_t {
        extern "C" fn dtor(ptr: *mut u16) {
            if !ptr.is_null() {
                unsafe {
                    U16CString::from_raw(ptr);
                }
            }
        }
        let str_value: String = s.into();
        let raw_value = U16CString::from_str(&str_value).unwrap().into_raw();
        chromium_sys::cef_string_utf16_t {
            length: str_value.len(),
            str_: raw_value,
            dtor: Some(dtor),
        }
    }

    // TODO: Rename to null?
    pub fn empty_raw() -> chromium_sys::cef_string_t {
        unsafe { std::mem::zeroed() }
    }
}
