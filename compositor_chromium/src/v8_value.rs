use std::os::raw::c_void;

use crate::{
    cef::V8ContextEntered,
    cef_ref::{CefRefData, CefStruct},
    cef_string::CefString,
    validated::{Validatable, Validated, ValidatedError},
};

/// Represents JavaScript values
pub struct V8Value {
    inner: Validated<chromium_sys::cef_v8value_t>,
}

impl V8Value {
    pub(crate) fn from_raw(v8_value: *mut chromium_sys::cef_v8value_t) -> Self {
        let inner = Validated::new(v8_value);
        Self { inner }
    }

    pub fn set_value_by_key(
        &mut self,
        key: &str,
        value: Self,
        attribute: V8PropertyAttribute,
    ) -> Result<bool, V8ValueError> {
        let key = CefString::new_raw(key);
        unsafe {
            let self_value = self.inner.get()?;
            let set_value = (*self_value).set_value_bykey.unwrap();
            Ok(set_value(self_value, &key, value.inner.get()?, attribute as u32) == 1)
        }
    }

    pub fn new_string(data: &str) -> Self {
        let data = CefString::new_raw(data);
        let inner = unsafe { chromium_sys::cef_v8value_create_string(&data) };

        Self::from_raw(inner)
    }

    /// Creates a new array buffer. It can be only created while in context.
    /// The buffer's memory is shared with V8 engine
    pub fn new_array_buffer(_context_entered: &V8ContextEntered, buffer: Vec<u8>) -> Self {
        let release_callback = V8ArrayBufferReleaseCallback {
            buffer_len: buffer.len(),
            buffer_cap: buffer.capacity(),
        };

        let buffer = buffer.leak();
        let inner = unsafe {
            chromium_sys::cef_v8value_create_array_buffer(
                buffer.as_mut_ptr() as *mut c_void,
                buffer.len(),
                CefRefData::new_ptr(release_callback),
            )
        };

        Self::from_raw(inner)
    }
}

struct V8ArrayBufferReleaseCallback {
    buffer_len: usize,
    buffer_cap: usize,
}

impl CefStruct for V8ArrayBufferReleaseCallback {
    type CefType = chromium_sys::cef_v8array_buffer_release_callback_t;

    fn cef_data(&self) -> Self::CefType {
        chromium_sys::cef_v8array_buffer_release_callback_t {
            base: unsafe { std::mem::zeroed() },
            release_buffer: Some(Self::release_buffer),
        }
    }

    fn base_mut(cef_data: &mut Self::CefType) -> &mut chromium_sys::cef_base_ref_counted_t {
        &mut cef_data.base
    }
}

impl V8ArrayBufferReleaseCallback {
    extern "C" fn release_buffer(
        self_: *mut chromium_sys::cef_v8array_buffer_release_callback_t,
        buffer: *mut c_void,
    ) {
        unsafe {
            let self_ref = CefRefData::<Self>::from_cef(self_);
            Vec::from_raw_parts(buffer, self_ref.buffer_len, self_ref.buffer_cap);
        }
    }
}

#[repr(u32)]
pub enum V8PropertyAttribute {
    None = chromium_sys::cef_v8_propertyattribute_t_V8_PROPERTY_ATTRIBUTE_NONE,
    ReadOnly = chromium_sys::cef_v8_propertyattribute_t_V8_PROPERTY_ATTRIBUTE_READONLY,
    DoNotEnum = chromium_sys::cef_v8_propertyattribute_t_V8_PROPERTY_ATTRIBUTE_DONTENUM,
    DoNotDelete = chromium_sys::cef_v8_propertyattribute_t_V8_PROPERTY_ATTRIBUTE_DONTDELETE,
}

impl Validatable for chromium_sys::cef_v8value_t {
    fn is_valid(&mut self) -> bool {
        let is_valid = self.is_valid.unwrap();
        unsafe { is_valid(self) == 1 }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum V8ValueError {
    #[error("V8Value is no longer valid")]
    NotValid(#[from] ValidatedError),
}
