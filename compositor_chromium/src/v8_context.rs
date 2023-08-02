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

    pub fn enter(&self) {
        unsafe {
            let enter_context = (*self.inner).enter.unwrap();
            enter_context(self.inner);
        }
    }

    pub fn exit(&self) {
        unsafe {
            let exit_context = (*self.inner).exit.unwrap();
            exit_context(self.inner);
        }
    }

    pub fn get_global(&self) -> V8Value {
        unsafe {
            let f = (*self.inner).get_global.unwrap();
            V8Value::new(f(self.inner))
        }
    }
}
