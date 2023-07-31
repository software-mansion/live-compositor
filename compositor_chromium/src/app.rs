use crate::{
    cef::{RenderProcessHandler, RenderProcessHandlerWrapper},
    cef_ref::{CefRefPtr, CefStruct},
    cef_string::CefString,
    command_line::CommandLine,
};

pub trait App {
    type RenderProcessHandlerType: RenderProcessHandler;

    fn on_before_command_line_processing(
        &mut self,
        process_type: String,
        command_line: &mut CommandLine,
    ) {
    }

    fn get_render_process_handler(&self) -> Option<Self::RenderProcessHandlerType> {
        None
    }
}

pub(crate) struct AppWrapper<A: App>(pub A);

impl<A: App> CefStruct for AppWrapper<A> {
    type CefType = chromium_sys::cef_app_t;

    fn get_cef_data(&self) -> Self::CefType {
        chromium_sys::cef_app_t {
            base: unsafe { std::mem::zeroed() },
            on_before_command_line_processing: Some(Self::on_before_command_line_processing),
            on_register_custom_schemes: None,
            get_resource_bundle_handler: None,
            get_browser_process_handler: None,
            get_render_process_handler: Some(Self::get_render_process_handler),
        }
    }

    fn get_base_mut(cef_data: &mut Self::CefType) -> &mut chromium_sys::cef_base_ref_counted_t {
        &mut cef_data.base
    }
}

impl<A: App> AppWrapper<A> {
    extern "C" fn on_before_command_line_processing(
        self_: *mut chromium_sys::cef_app_t,
        process_type: *const chromium_sys::cef_string_t,
        command_line: *mut chromium_sys::cef_command_line_t,
    ) {
        let self_ref = unsafe { CefRefPtr::<Self>::from_cef(self_) };
        let mut command_line = CommandLine(command_line);
        let process_type = CefString::from_raw(process_type);
        self_ref
            .0
            .on_before_command_line_processing(process_type, &mut command_line);
    }

    extern "C" fn get_render_process_handler(
        self_: *mut chromium_sys::cef_app_t,
    ) -> *mut chromium_sys::cef_render_process_handler_t {
        let self_ref = unsafe { CefRefPtr::<Self>::from_cef(self_) };
        match self_ref.0.get_render_process_handler() {
            Some(handler) => CefRefPtr::new(RenderProcessHandlerWrapper(handler)),
            None => std::ptr::null_mut(),
        }
    }
}
