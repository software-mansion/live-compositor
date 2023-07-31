use std::marker::PhantomData;

use crate::cef::{ProcessId, ProcessMessage};

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
            let send_message = (&mut *self.inner).send_process_message.unwrap();
            send_message(self.inner, pid as u32, msg.inner);
        }
    }
}
