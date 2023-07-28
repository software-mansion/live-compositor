use std::{ops::Deref, os::raw::c_int};

use crate::cef_string::CefString;

#[derive(Default)]
pub struct Settings {
    /// If set to `true` message loop can run on separate thread
    /// Not supported by MacOS
    pub multi_threaded_message_loop: bool,
    pub external_message_pump: bool,
    pub windowless_rendering_enabled: bool,
    pub log_severity: LogSeverity,
    pub remote_debugging_port: u16,
    pub background_color: u32,
}

impl Settings {
    pub fn into_raw(self) -> chromium_sys::cef_settings_t {
        chromium_sys::cef_settings_t {
            size: std::mem::size_of::<chromium_sys::cef_settings_t>(),
            no_sandbox: true as c_int,
            browser_subprocess_path: CefString::empty_raw(),
            framework_dir_path: CefString::empty_raw(),
            main_bundle_path: CefString::empty_raw(),
            chrome_runtime: false as c_int,
            multi_threaded_message_loop: self.multi_threaded_message_loop as c_int,
            external_message_pump: self.external_message_pump as c_int,
            windowless_rendering_enabled: self.windowless_rendering_enabled as c_int,
            command_line_args_disabled: false as c_int,
            cache_path: CefString::empty_raw(),
            root_cache_path: CefString::empty_raw(),
            persist_session_cookies: false as c_int,
            persist_user_preferences: false as c_int,
            user_agent: CefString::empty_raw(),
            user_agent_product: CefString::empty_raw(),
            locale: CefString::empty_raw(),
            log_file: CefString::empty_raw(),
            log_severity: self.log_severity as u32,
            javascript_flags: CefString::empty_raw(),
            resources_dir_path: CefString::empty_raw(),
            locales_dir_path: CefString::empty_raw(),
            pack_loading_disabled: false as c_int,
            remote_debugging_port: self.remote_debugging_port as c_int,
            uncaught_exception_stack_size: 0 as c_int,
            background_color: self.background_color,
            accept_language_list: CefString::empty_raw(),
            cookieable_schemes_list: CefString::empty_raw(),
            cookieable_schemes_exclude_defaults: false as c_int,
        }
    }
}

#[repr(u32)]
pub enum LogSeverity {
    Default = chromium_sys::cef_log_severity_t_LOGSEVERITY_DEFAULT,
    Debug = chromium_sys::cef_log_severity_t_LOGSEVERITY_DEBUG,
    Info = chromium_sys::cef_log_severity_t_LOGSEVERITY_INFO,
    Warning = chromium_sys::cef_log_severity_t_LOGSEVERITY_WARNING,
    Error = chromium_sys::cef_log_severity_t_LOGSEVERITY_ERROR,
    Fatal = chromium_sys::cef_log_severity_t_LOGSEVERITY_FATAL,
    Disable = chromium_sys::cef_log_severity_t_LOGSEVERITY_DISABLE,
}

impl Default for LogSeverity {
    fn default() -> Self {
        Self::Default
    }
}
