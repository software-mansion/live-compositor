use std::marker::PhantomData;

use crate::{cef_string::CefString, frame::Frame};

pub struct Browser<'a> {
    inner: *mut chromium_sys::cef_browser_t,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> Browser<'a> {
    pub(crate) fn new(browser: *mut chromium_sys::cef_browser_t) -> Self {
        Self {
            inner: browser,
            _lifetime: PhantomData,
        }
    }

    pub fn is_loading(&self) -> bool {
        unsafe {
            let browser = &mut *self.inner;
            let f = browser.is_loading.unwrap();
            f(self.inner) == 1
        }
    }

    pub fn get_main_frame(&self) -> Frame<'a> {
        unsafe {
            let browser = &mut *self.inner;
            let f = browser.get_main_frame.unwrap();
            Frame::new(f(self.inner))
        }
    }
}

#[derive(Default)]
pub struct BrowserSettings {
    pub windowless_frame_rate: i32,
}

impl BrowserSettings {
    pub(crate) fn into_raw(self) -> chromium_sys::cef_browser_settings_t {
        chromium_sys::_cef_browser_settings_t {
            size: std::mem::size_of::<chromium_sys::cef_browser_settings_t>(),
            windowless_frame_rate: self.windowless_frame_rate,
            standard_font_family: CefString::empty_raw(),
            fixed_font_family: CefString::empty_raw(),
            serif_font_family: CefString::empty_raw(),
            sans_serif_font_family: CefString::empty_raw(),
            cursive_font_family: CefString::empty_raw(),
            fantasy_font_family: CefString::empty_raw(),
            default_font_size: 0,
            default_fixed_font_size: 0,
            minimum_font_size: 0,
            minimum_logical_font_size: 0,
            default_encoding: CefString::empty_raw(),
            remote_fonts: chromium_sys::cef_state_t_STATE_DEFAULT,
            javascript: chromium_sys::cef_state_t_STATE_DEFAULT,
            javascript_close_windows: chromium_sys::cef_state_t_STATE_DEFAULT,
            javascript_access_clipboard: chromium_sys::cef_state_t_STATE_DEFAULT,
            javascript_dom_paste: chromium_sys::cef_state_t_STATE_DEFAULT,
            image_loading: chromium_sys::cef_state_t_STATE_DEFAULT,
            image_shrink_standalone_to_fit: chromium_sys::cef_state_t_STATE_DEFAULT,
            text_area_resize: chromium_sys::cef_state_t_STATE_DEFAULT,
            tab_to_links: chromium_sys::cef_state_t_STATE_DEFAULT,
            local_storage: chromium_sys::cef_state_t_STATE_DEFAULT,
            databases: chromium_sys::cef_state_t_STATE_DEFAULT,
            webgl: chromium_sys::cef_state_t_STATE_DEFAULT,
            background_color: 0xFFFFFF00,
            accept_language_list: CefString::empty_raw(),
            chrome_status_bubble: chromium_sys::cef_state_t_STATE_DEFAULT,
        }
    }
}
