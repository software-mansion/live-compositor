use crate::{
    cef::Browser,
    cef_ref::{CefRefPtr, CefStruct},
};

pub trait RenderProcessHandler {
    fn on_context_created(&mut self, browser: Browser<'_>) {}
}

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
            on_process_message_received: None,
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
        self_ref.0.on_context_created(browser);
    }
}
