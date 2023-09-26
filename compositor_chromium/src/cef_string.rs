use widestring::U16CString;

/// Helper for handling UTF-16 `cef_string_t`
pub struct CefString;

impl CefString {
    pub fn new_raw<S: Into<String>>(s: S) -> chromium_sys::cef_string_t {
        extern "C" fn dtor(ptr: *mut u16) {
            if !ptr.is_null() {
                unsafe {
                    let _ = U16CString::from_raw(ptr);
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

    /// Returns Rust's `String` from UTF-16 `cef_string_t`.
    /// If `ptr` is null, empty string is returned
    pub fn from_raw(ptr: *const chromium_sys::cef_string_t) -> String {
        if ptr.is_null() {
            return String::new();
        }

        unsafe {
            let cef_str = *ptr;
            U16CString::from_ptr(cef_str.str_, cef_str.length)
                .unwrap()
                .to_string_lossy()
        }
    }

    pub fn from_userfree(ptr: chromium_sys::cef_string_userfree_utf16_t) -> String {
        let cef_string = CefString::from_raw(ptr);
        unsafe {
            chromium_sys::cef_string_userfree_utf16_free(ptr);
        }

        cef_string
    }

    /// Creates a new empty `cef_string_t`
    pub fn empty_raw() -> chromium_sys::cef_string_t {
        unsafe { std::mem::zeroed() }
    }
}
