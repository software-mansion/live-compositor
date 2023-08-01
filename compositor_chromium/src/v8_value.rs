use std::{marker::PhantomData, os::raw::c_void};

use crate::{
    cef_ref::{CefRefPtr, CefStruct},
    cef_string::CefString,
};

pub struct V8Value<'a> {
    pub(crate) inner: *mut chromium_sys::cef_v8value_t,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> V8Value<'a> {
    pub(crate) fn new(v8_value: *mut chromium_sys::cef_v8value_t) -> Self {
        Self {
            inner: v8_value,
            _lifetime: PhantomData,
        }
    }

    pub fn set_value_by_key(
        &mut self,
        key: &str,
        value: Self,
        attribute: V8PropertyAttribute,
    ) -> bool {
        let key = CefString::new_raw(key);
        unsafe {
            let set_value = (&mut *self.inner).set_value_bykey.unwrap();
            set_value(self.inner, &key, value.inner, attribute as u32) == 1
        }
    }

    pub fn create_string(data: &str) -> Self {
        let data = CefString::new_raw(data);
        let inner = unsafe { chromium_sys::cef_v8value_create_string(&data) };

        Self::new(inner)
    }

    pub fn create_array_buffer(buffer: Vec<u8>) -> Self {
        let release_callback = V8ArrayBufferReleaseCallback {
            buffer_len: buffer.len(),
            buffer_cap: buffer.capacity(),
        };

        let buffer = buffer.leak();
        let inner = unsafe {
            chromium_sys::cef_v8value_create_array_buffer(
                buffer.as_mut_ptr() as *mut c_void,
                buffer.len(),
                CefRefPtr::new(release_callback),
            )
        };

        Self::new(inner)
    }
}

struct V8ArrayBufferReleaseCallback {
    buffer_len: usize,
    buffer_cap: usize,
}

impl CefStruct for V8ArrayBufferReleaseCallback {
    type CefType = chromium_sys::cef_v8array_buffer_release_callback_t;

    fn get_cef_data(&self) -> Self::CefType {
        chromium_sys::cef_v8array_buffer_release_callback_t {
            base: unsafe { std::mem::zeroed() },
            release_buffer: Some(Self::release_buffer),
        }
    }

    fn get_base_mut(cef_data: &mut Self::CefType) -> &mut chromium_sys::cef_base_ref_counted_t {
        &mut cef_data.base
    }
}

impl V8ArrayBufferReleaseCallback {
    extern "C" fn release_buffer(
        self_: *mut chromium_sys::cef_v8array_buffer_release_callback_t,
        buffer: *mut c_void,
    ) {
        unsafe {
            let self_ref = CefRefPtr::<Self>::from_cef(self_);
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
