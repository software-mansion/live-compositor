pub trait RenderHandler {
    fn get_view_rect(
        &mut self,
        browser: &mut chromium_sys::cef_browser_t,
        rect: &mut chromium_sys::cef_rect_t,
    );

    fn on_paint(&mut self, buffer: &[u8], width: u32, height: u32);
}

pub(crate) struct RenderHandlerWrapper<T: RenderHandler>(pub T);

impl<T: RenderHandler> RenderHandlerWrapper<T> {
    extern "C" fn get_view_rect(
        self_: *mut chromium_sys::cef_render_handler_t,
        browser: *mut chromium_sys::cef_browser_t,
        rect: *mut chromium_sys::cef_rect_t,
    ) {
        // handler.get_view_rect(browser, rect);
    }
    // TODO: use ref count
    pub fn into_raw(&mut self) -> chromium_sys::cef_render_handler_t {
        chromium_sys::cef_render_handler_t {
            base: todo!(),
            get_accessibility_handler: None,
            get_root_screen_rect: None,
            get_view_rect: Some(Self::get_view_rect),
            get_screen_point: todo!(),
            get_screen_info: todo!(),
            on_popup_show: todo!(),
            on_popup_size: todo!(),
            on_paint: todo!(),
            on_accelerated_paint: todo!(),
            get_touch_handle_size: todo!(),
            on_touch_handle_state_changed: todo!(),
            start_dragging: todo!(),
            update_drag_cursor: todo!(),
            on_scroll_offset_changed: todo!(),
            on_ime_composition_range_changed: todo!(),
            on_text_selection_changed: todo!(),
            on_virtual_keyboard_requested: todo!(),
        }
    }
}
