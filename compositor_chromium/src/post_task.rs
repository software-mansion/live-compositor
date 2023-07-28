use chromium_sys::_cef_task_t;

use crate::cef_ref::{CefRefPtr, CefStruct};

pub struct Task<F: FnOnce()> {
    inner: Option<F>,
}

impl<F: FnOnce()> CefStruct for Task<F> {
    type CefType = chromium_sys::cef_task_t;

    fn get_cef_data(&self) -> Self::CefType {
        chromium_sys::cef_task_t {
            base: unsafe { std::mem::zeroed() },
            execute: Some(Self::execute),
        }
    }

    fn get_base_mut(cef_data: &mut Self::CefType) -> &mut chromium_sys::cef_base_ref_counted_t {
        &mut cef_data.base
    }
}

impl<F: FnOnce()> Task<F> {
    pub fn new(task: F) -> Self {
        Self { inner: Some(task) }
    }

    pub fn run(self, thread_id: chromium_sys::cef_thread_id_t) {
        let task = CefRefPtr::new(self);
        unsafe {
            chromium_sys::cef_post_task(thread_id, task);
        }
    }

    extern "C" fn execute(self_: *mut _cef_task_t) {
        let self_ref = unsafe { CefRefPtr::<Self>::from_cef(self_) };
        if let Some(task) = self_ref.inner.take() {
            task();
        }
    }
}
