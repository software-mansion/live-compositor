use std::{
    os::raw::c_int,
    sync::atomic::{AtomicUsize, Ordering},
};

pub(crate) trait CefRef {
    type CefType;

    fn to_cef(self) -> Self::CefType;

    fn ref_base_mut(&mut self) -> &mut chromium_sys::cef_base_ref_counted_t;
}

#[repr(C)]
pub(crate) struct CefRefPtr<T: CefRef> {
    data: T,
    ref_count: AtomicUsize,
}

impl<T: CefRef> CefRefPtr<T> {
    // Taken from std::sync::Arc
    const MAX_REFCOUNT: usize = (isize::MAX) as usize;

    pub fn new(mut data: T) -> Self {
        let base = data.ref_base_mut();
        *base = chromium_sys::cef_base_ref_counted_t {
            size: std::mem::size_of::<Self>(),
            add_ref: Some(Self::add_ref),
            release: Some(Self::release),
            has_one_ref: Some(Self::has_one_ref),
            has_at_least_one_ref: Some(Self::has_at_least_one_ref),
        };

        Self {
            data,
            ref_count: AtomicUsize::new(1),
        }
    }

    unsafe fn from_base<'a>(base: *mut chromium_sys::cef_base_ref_counted_t) -> &'a mut Self {
        unsafe { &mut *(base as *mut Self) }
    }

    extern "C" fn add_ref(base: *mut chromium_sys::cef_base_ref_counted_t) {
        let self_ptr = unsafe { Self::from_base(base) };
        let old_count = self_ptr.ref_count.fetch_add(1, Ordering::Relaxed);
        if old_count > Self::MAX_REFCOUNT {
            panic!("Reached max ref count limit");
        }
    }

    extern "C" fn release(base: *mut chromium_sys::cef_base_ref_counted_t) -> c_int {
        let self_ptr = unsafe { Self::from_base(base) };
        let old_count = self_ptr.ref_count.fetch_sub(1, Ordering::Release);
        std::sync::atomic::fence(Ordering::Acquire);

        let should_drop = old_count == 1;
        if should_drop {
            unsafe {
                Box::from_raw(self_ptr);
            }
        }

        should_drop as c_int
    }

    extern "C" fn has_one_ref(base: *mut chromium_sys::cef_base_ref_counted_t) -> c_int {
        let self_ptr = unsafe { Self::from_base(base) };
        let is_one_ref = (self_ptr.ref_count.load(Ordering::SeqCst)) == 1;

        is_one_ref as c_int
    }

    extern "C" fn has_at_least_one_ref(base: *mut chromium_sys::cef_base_ref_counted_t) -> c_int {
        let self_ptr = unsafe { Self::from_base(base) };
        let has_any_refs = (self_ptr.ref_count.load(Ordering::SeqCst)) >= 1;

        has_any_refs as c_int
    }
}
