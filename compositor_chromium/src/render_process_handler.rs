use std::os::raw::c_int;

use crate::{
    cef::{Browser, Frame, ProcessId, ProcessMessage, V8Context},
    cef_ref::{CefRefPtr, CefStruct},
};

pub trait RenderProcessHandler {
    fn on_context_created(
        &mut self,
        browser: Browser<'_>,
        frame: Frame<'_>,
        context: V8Context<'_>,
    ) {
    }

    fn on_process_message_received(
        &mut self,
        browser: Browser<'_>,
        frame: Frame<'_>,
        source_process: ProcessId,
        message: ProcessMessage,
    ) -> bool {
        false
    }
}

// TODO: Rewrite this
impl RenderProcessHandler for () {}

pub(crate) struct RenderProcessHandlerWrapper<R: RenderProcessHandler>(pub R);

impl<R: RenderProcessHandler> CefStruct for RenderProcessHandlerWrapper<R> {
    type CefType = chromium_sys::cef_render_process_handler_t;

    fn get_cef_data(&self) -> Self::CefType {
        chromium_sys::cef_render_process_handler_t {
            base: unsafe { std::mem::zeroed() },
            on_web_kit_initialized: None,
            on_browser_created: None,
            on_browser_destroyed: None,
            get_load_handler: None,
            on_context_created: Some(Self::on_context_created),
            on_context_released: None,
            on_uncaught_exception: None,
            on_focused_node_changed: None,
            on_process_message_received: Some(Self::on_process_message_received),
        }
    }

    fn get_base_mut(cef_data: &mut Self::CefType) -> &mut chromium_sys::cef_base_ref_counted_t {
        &mut cef_data.base
    }
}

impl<R: RenderProcessHandler> RenderProcessHandlerWrapper<R> {
    extern "C" fn on_context_created(
        self_: *mut chromium_sys::cef_render_process_handler_t,
        browser: *mut chromium_sys::cef_browser_t,
        frame: *mut chromium_sys::cef_frame_t,
        context: *mut chromium_sys::cef_v8context_t,
    ) {
        let self_ref = unsafe { CefRefPtr::<Self>::from_cef(self_) };
        let browser = Browser::new(browser);
        let frame = Frame::new(frame);
        let v8_context = V8Context::new(context);
        self_ref.0.on_context_created(browser, frame, v8_context);
    }

    extern "C" fn on_process_message_received(
        self_: *mut chromium_sys::cef_render_process_handler_t,
        browser: *mut chromium_sys::cef_browser_t,
        frame: *mut chromium_sys::cef_frame_t,
        source_process: chromium_sys::cef_process_id_t,
        message: *mut chromium_sys::cef_process_message_t,
    ) -> c_int {
        let self_ref = unsafe { CefRefPtr::<Self>::from_cef(self_) };
        let browser = Browser::new(browser);
        let frame = Frame::new(frame);
        let message = ProcessMessage { inner: message };

        let is_handled =
            self_ref
                .0
                .on_process_message_received(browser, frame, source_process.into(), message);

        is_handled as c_int
    }
}
