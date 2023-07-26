use std::os::raw::{c_int, c_void};

use crate::cef_ref::{CefRefPtr, CefStruct};

pub trait RenderHandler {
    // TODO: Implement Rect
    fn get_view_rect(&mut self, rect: &mut chromium_sys::cef_rect_t);

    // TODO: Use compositor_common::Resolution
    fn on_paint(&mut self, buffer: &[u8], width: i32, height: i32);
}

pub(crate) struct RenderHandlerWrapper<R: RenderHandler>(pub R);

impl<R: RenderHandler> CefStruct for RenderHandlerWrapper<R> {
    type CefType = chromium_sys::cef_render_handler_t;

    fn get_cef_data(&self) -> Self::CefType {
        chromium_sys::cef_render_handler_t {
            base: unsafe { std::mem::zeroed() },
            get_accessibility_handler: None,
            get_root_screen_rect: None,
            get_view_rect: Some(Self::get_view_rect),
            get_screen_point: None,
            get_screen_info: None,
            on_popup_show: None,
            on_popup_size: None,
            on_paint: Some(Self::on_paint),
            on_accelerated_paint: None,
            get_touch_handle_size: None,
            on_touch_handle_state_changed: None,
            start_dragging: None,
            update_drag_cursor: None,
            on_scroll_offset_changed: None,
            on_ime_composition_range_changed: None,
            on_text_selection_changed: None,
            on_virtual_keyboard_requested: None,
        }
    }

    fn get_base_mut(cef_data: &mut Self::CefType) -> &mut chromium_sys::cef_base_ref_counted_t {
        &mut cef_data.base
    }
}

impl<R: RenderHandler> RenderHandlerWrapper<R> {
    extern "C" fn get_view_rect(
        self_: *mut chromium_sys::cef_render_handler_t,
        browser: *mut chromium_sys::cef_browser_t,
        rect: *mut chromium_sys::cef_rect_t,
    ) {
        unsafe {
            let mut self_ref = CefRefPtr::<Self>::from_cef(self_);
            self_ref.0.get_view_rect(rect.as_mut().unwrap());
        }
    }

    extern "C" fn on_paint(
        self_: *mut chromium_sys::cef_render_handler_t,
        browser: *mut chromium_sys::cef_browser_t,
        type_: chromium_sys::cef_paint_element_type_t,
        dirty_rects_count: usize,
        dirt_rects: *const chromium_sys::cef_rect_t,
        buffer: *const c_void,
        width: c_int,
        height: c_int,
    ) {
        unsafe {
            let mut self_ref = CefRefPtr::<Self>::from_cef(self_);
            let buffer =
                std::slice::from_raw_parts(buffer as *const u8, (4 * width * height) as usize);
            self_ref.0.on_paint(buffer, width, height);
        }
    }
}
