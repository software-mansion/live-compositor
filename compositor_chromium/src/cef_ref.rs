use std::{
    ops::{Deref, DerefMut},
    os::raw::c_int,
    sync::atomic::{AtomicUsize, Ordering},
};

pub(crate) trait CefStruct {
    type CefType;

    fn get_cef_data(&self) -> Self::CefType;

    fn get_base_mut(cef_data: &mut Self::CefType) -> &mut chromium_sys::cef_base_ref_counted_t;
}

// Each CEF struct with ref counting capability has a base struct as a first field
// This lets us simulate inheritance-like behavior
// T::CefType represents CEF struct which has ref counting capability
// http://www.deleveld.dds.nl/inherit.htm
#[repr(C)]
pub(crate) struct CefRefPtr<T: CefStruct> {
    cef_data: T::CefType,
    rust_data: T,
    ref_count: AtomicUsize,
}

impl<T: CefStruct> Deref for CefRefPtr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.rust_data
    }
}

impl<T: CefStruct> DerefMut for CefRefPtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.rust_data
    }
}

impl<T: CefStruct> CefRefPtr<T> {
    // Taken from std::sync::Arc
    const MAX_REFCOUNT: usize = (isize::MAX) as usize;

    pub fn new(mut data: T) -> *mut T::CefType {
        let mut cef_data = data.get_cef_data();

        // Init ref counting for T::CefType
        let base = T::get_base_mut(&mut cef_data);
        *base = chromium_sys::cef_base_ref_counted_t {
            size: std::mem::size_of::<Self>(),
            add_ref: Some(Self::add_ref),
            release: Some(Self::release),
            has_one_ref: Some(Self::has_one_ref),
            has_at_least_one_ref: Some(Self::has_at_least_one_ref),
        };

        let cef_ref = Self {
            cef_data,
            rust_data: data,
            ref_count: AtomicUsize::new(1),
        };

        Box::into_raw(Box::new(cef_ref)) as *mut T::CefType
    }

    pub unsafe fn from_cef<'a>(cef_data: *mut T::CefType) -> &'a mut Self {
        unsafe { &mut *(cef_data as *mut Self) }
    }

    unsafe fn from_base<'a>(base: *mut chromium_sys::cef_base_ref_counted_t) -> &'a mut Self {
        unsafe { &mut *(base as *mut Self) }
    }

    extern "C" fn add_ref(base: *mut chromium_sys::cef_base_ref_counted_t) {
        let self_ref = unsafe { Self::from_base(base) };
        let old_count = self_ref.ref_count.fetch_add(1, Ordering::Relaxed);
        if old_count > Self::MAX_REFCOUNT {
            panic!("Reached max ref count limit");
        }
    }

    extern "C" fn release(base: *mut chromium_sys::cef_base_ref_counted_t) -> c_int {
        let self_ref = unsafe { Self::from_base(base) };
        let old_count = self_ref.ref_count.fetch_sub(1, Ordering::Release);
        std::sync::atomic::fence(Ordering::Acquire);

        let should_drop = old_count == 1;
        if should_drop {
            unsafe {
                // Load raw Self instance and let Box drop it
                Box::from_raw(self_ref);
            }
        }

        should_drop as c_int
    }

    extern "C" fn has_one_ref(base: *mut chromium_sys::cef_base_ref_counted_t) -> c_int {
        let self_ref = unsafe { Self::from_base(base) };
        let is_one_ref = (self_ref.ref_count.load(Ordering::SeqCst)) == 1;

        is_one_ref as c_int
    }

    extern "C" fn has_at_least_one_ref(base: *mut chromium_sys::cef_base_ref_counted_t) -> c_int {
        let self_ref = unsafe { Self::from_base(base) };
        let has_any_refs = (self_ref.ref_count.load(Ordering::SeqCst)) >= 1;

        has_any_refs as c_int
    }
}
