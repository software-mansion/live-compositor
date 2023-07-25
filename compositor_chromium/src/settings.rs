use std::{ops::Deref, os::raw::c_int};

use crate::cef_string::CefString;

pub struct Settings(chromium_sys::cef_settings_t);

impl Settings {
    pub fn raw(&self) -> &chromium_sys::cef_settings_t {
        return &self.0;
    }
}

pub struct SettingsBuilder {
    // TODO: Fill in
}

// TODO: add builder methods
impl SettingsBuilder {
    pub fn new() -> Self {
        Self {}
    }

    pub fn build(self) -> Settings {
        let inner = chromium_sys::cef_settings_t {
            size: std::mem::size_of::<chromium_sys::cef_settings_t>(),
            no_sandbox: true as c_int,
            browser_subprocess_path: CefString::empty_raw(),
            framework_dir_path: CefString::empty_raw(),
            main_bundle_path: CefString::empty_raw(),
            chrome_runtime: false as c_int,
            multi_threaded_message_loop: false as c_int,
            external_message_pump: false as c_int,
            windowless_rendering_enabled: true as c_int,
            command_line_args_disabled: false as c_int,
            cache_path: CefString::empty_raw(),
            root_cache_path: CefString::empty_raw(),
            persist_session_cookies: false as c_int,
            persist_user_preferences: false as c_int,
            user_agent: CefString::empty_raw(),
            user_agent_product: CefString::empty_raw(),
            locale: CefString::empty_raw(),
            log_file: CefString::empty_raw(),
            // TODO: Add log severity enum
            log_severity: chromium_sys::cef_log_severity_t_LOGSEVERITY_INFO,
            javascript_flags: CefString::empty_raw(),
            resources_dir_path: CefString::empty_raw(),
            locales_dir_path: CefString::empty_raw(),
            pack_loading_disabled: false as c_int,
            remote_debugging_port: 9000 as c_int,
            uncaught_exception_stack_size: 0 as c_int,
            background_color: 0xFFFFFF00,
            accept_language_list: CefString::empty_raw(),
            cookieable_schemes_list: CefString::empty_raw(),
            cookieable_schemes_exclude_defaults: false as c_int,
        };

        Settings(inner)
    }
}
