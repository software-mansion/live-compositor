use crate::{cef_string::CefString, validated::Validated};

use super::value::{V8Value, V8ValueError};

pub struct V8String(pub(super) Validated<chromium_sys::cef_v8value_t>);

impl V8String {
    pub fn new(value: &str) -> Self {
        let value = CefString::new_raw(value);
        let inner = unsafe { chromium_sys::cef_v8value_create_string(&value) };

        Self(Validated::new(inner))
    }

    pub fn get(&self) -> Result<String, V8ValueError> {
        let inner = self.0.get()?;
        unsafe {
            let get_value = (*inner).get_string_value.unwrap();
            let value = get_value(inner);
            Ok(CefString::from_userfree(value))
        }
    }
}

impl From<V8String> for V8Value {
    fn from(value: V8String) -> Self {
        Self::String(value)
    }
}
