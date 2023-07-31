use std::marker::PhantomData;

use crate::cef::V8Value;

pub struct V8Context<'a> {
    inner: *mut chromium_sys::cef_v8context_t,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> V8Context<'a> {
    pub(crate) fn new(v8_context: *mut chromium_sys::cef_v8context_t) -> Self {
        Self {
            inner: v8_context,
            _lifetime: PhantomData,
        }
    }

    pub fn get_global(&self) -> V8Value {
        unsafe {
            let f = (&mut *self.inner).get_global.unwrap();
            V8Value::new(f(self.inner))
        }
    }
}
