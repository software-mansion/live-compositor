use crate::validated::Validated;

use super::value::{V8Value, V8ValueError};

pub struct V8Int(pub(super) Validated<chromium_sys::cef_v8value_t>);

impl V8Int {
    pub fn new(value: i32) -> Self {
        let inner = unsafe { chromium_sys::cef_v8value_create_int(value) };
        Self(Validated(inner))
    }

    pub fn get(&self) -> Result<i32, V8ValueError> {
        let inner = self.0.get()?;
        unsafe {
            let get_value = (*inner).get_int_value.unwrap();
            Ok(get_value(inner))
        }
    }
}

impl From<V8Int> for V8Value {
    fn from(value: V8Int) -> Self {
        Self::Int(value)
    }
}
