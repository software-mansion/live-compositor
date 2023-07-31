use std::{marker::PhantomData, os::raw::c_void};

use crate::cef_ref::{CefRefPtr, CefStruct};

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

        Self {
            inner,
            _lifetime: PhantomData,
        }
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
