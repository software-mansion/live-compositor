use std::os::raw::c_int;

use crate::cef_string::CefString;

pub struct WindowInfo {
    pub hidden: bool,
    pub windowless_rendering_enabled: bool,
}

impl WindowInfo {
    pub(crate) fn into_raw(self) -> chromium_sys::cef_window_info_t {
        chromium_sys::cef_window_info_t {
            window_name: CefString::empty_raw(),
            bounds: unsafe { std::mem::zeroed() },
            hidden: self.hidden as c_int,
            parent_view: std::ptr::null_mut(),
            windowless_rendering_enabled: self.windowless_rendering_enabled as c_int,
            shared_texture_enabled: false as c_int,
            external_begin_frame_enabled: false as c_int,
            view: std::ptr::null_mut(),
        }
    }
}
