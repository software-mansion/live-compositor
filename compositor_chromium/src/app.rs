use crate::cef_ref::{CefRefPtr, CefStruct};

pub trait App {
    fn on_before_command_line_processing(
        &mut self,
        // TODO: Implement CommandLine
        // process_type: &str,
        // command_line: &mut chromium_sys::cef_command_line_t,
    ) {
    }
}

pub(crate) struct AppWrapper<T: App>(pub T);

impl<T: App> CefStruct for AppWrapper<T> {
    type CefType = chromium_sys::cef_app_t;

    fn get_cef_data(&self) -> Self::CefType {
        chromium_sys::cef_app_t {
            base: unsafe { std::mem::zeroed() },
            on_before_command_line_processing: Some(Self::on_before_command_line_processing),
            on_register_custom_schemes: None,
            get_resource_bundle_handler: None,
            get_browser_process_handler: None,
            get_render_process_handler: None,
        }
    }

    fn get_base_mut(cef_data: &mut Self::CefType) -> &mut chromium_sys::cef_base_ref_counted_t {
        &mut cef_data.base
    }
}

impl<T: App> AppWrapper<T> {
    extern "C" fn on_before_command_line_processing(
        self_: *mut chromium_sys::cef_app_t,
        process_type: *const chromium_sys::cef_string_t,
        command_line: *mut chromium_sys::cef_command_line_t,
    ) {
        unsafe {
            let mut self_ref = CefRefPtr::<Self>::from_cef(self_);
            self_ref.0.on_before_command_line_processing();
        }
    }
}
