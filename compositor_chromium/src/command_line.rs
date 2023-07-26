use crate::cef_string::CefString;

pub struct CommandLine(pub(crate) *mut chromium_sys::cef_command_line_t);

impl CommandLine {
    pub fn append_switch(&mut self, name: &str) {
        let name = CefString::new_raw(name);
        unsafe {
            let cmd = &mut *self.0;
            let f = cmd.append_switch.unwrap();
            f(self.0, &name);
        }
    }

    pub fn append_switch_with_value(&mut self, name: &str, value: &str) {
        let name = CefString::new_raw(name);
        let value = CefString::new_raw(value);
        unsafe {
            let cmd = &mut *self.0;
            let f = cmd.append_switch_with_value.unwrap();
            f(self.0, &name, &value);
        }
    }

    pub fn has_switch(&self, name: &str) -> bool {
        let name = CefString::new_raw(name);
        unsafe {
            let cmd = &mut *self.0;
            let f = cmd.has_switch.unwrap();
            f(self.0, &name) == 1
        }
    }
}
