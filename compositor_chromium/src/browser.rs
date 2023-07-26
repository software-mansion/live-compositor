use crate::{
    cef_ref::CefRefPtr,
    cef_string::CefString,
    client::{Client, ClientWrapper},
    window_info::{self, WindowInfo},
};

pub struct Browser {
    pub(crate) client: *mut chromium_sys::cef_client_t,
    pub(crate) window_info: chromium_sys::cef_window_info_t,
    pub(crate) settings: chromium_sys::cef_browser_settings_t,
}

impl Browser {
    pub fn new<C: Client>(client: C, window_info: WindowInfo, settings: BrowserSettings) -> Self {
        let client = CefRefPtr::new(ClientWrapper(client));
        let window_info = window_info.into_raw();
        let settings = settings.into_raw();

        Self {
            client,
            window_info,
            settings,
        }
    }
}

#[derive(Default)]
pub struct BrowserSettings {
    pub windowless_frame_rate: i32,
}

impl BrowserSettings {
    fn into_raw(self) -> chromium_sys::cef_browser_settings_t {
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
