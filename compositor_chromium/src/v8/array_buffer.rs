use std::os::raw::c_void;

use crate::{
    cef_ref::{CefRefData, CefStruct},
    validated::Validated,
};

use super::{context::V8ContextEntered, value::V8Value};

pub struct V8ArrayBuffer(pub(super) Validated<chromium_sys::cef_v8value_t>);

impl V8ArrayBuffer {
    pub fn new(buffer: Vec<u8>, _context_entered: &V8ContextEntered) -> Self {
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

        Self(Validated(inner))
    }
}

impl From<V8ArrayBuffer> for V8Value {
    fn from(value: V8ArrayBuffer) -> Self {
        Self::ArrayBuffer(value)
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
