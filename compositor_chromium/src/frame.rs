use std::marker::PhantomData;

use crate::cef::{ProcessId, ProcessMessage, V8Context, ThreadId};

pub struct Frame<'a> {
    inner: *mut chromium_sys::cef_frame_t,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> Frame<'a> {
    pub(crate) fn new(frame: *mut chromium_sys::cef_frame_t) -> Self {
        Self {
            inner: frame,
            _lifetime: PhantomData,
        }
    }

    pub fn send_process_message(&self, pid: ProcessId, msg: ProcessMessage) {
        unsafe {
            let send_message = (*self.inner).send_process_message.unwrap();
            send_message(self.inner, pid as u32, msg.inner);
        }
    }

    /// Can be only called from the renderer process
    pub fn get_v8_context(&self) -> Option<V8Context<'a>> {
        unsafe {
            if chromium_sys::cef_currently_on(ThreadId::Renderer as u32) != 1 {
                return None;
            }

            let get_v8_context = (*self.inner).get_v8context.unwrap();
            let context = get_v8_context(self.inner);
            if context.is_null() {
                return None;
            }
            Some(V8Context::new(context))
        }
    }
}
