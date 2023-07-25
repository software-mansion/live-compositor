pub trait App {
    fn on_before_command_line_processing(
        &mut self,
        process_type: &str,
        command_line: &chromium_sys::cef_command_line_t,
    ) {
    }
}
