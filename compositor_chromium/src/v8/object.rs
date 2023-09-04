use crate::{cef_string::CefString, validated::Validated};

use super::value::{V8Value, V8ValueError};

pub struct V8Object(pub(super) Validated<chromium_sys::cef_v8value_t>);

impl V8Object {
    pub fn has(&self, key: &str) -> Result<bool, V8ValueError> {
        let inner = self.0.get()?;
        let key = CefString::new_raw(key);
        unsafe {
            let has_value = (*inner).has_value_bykey.unwrap();
            Ok(has_value(inner, &key) == 1)
        }
    }

    /// Returns `true` if value was set successfully
    pub fn set(
        &mut self,
        key: &str,
        value: &V8Value,
        attribute: V8PropertyAttribute,
    ) -> Result<bool, V8ValueError> {
        let inner = self.0.get()?;
        let key = CefString::new_raw(key);
        unsafe {
            let set_value = (*inner).set_value_bykey.unwrap();
            Ok(set_value(inner, &key, value.get_raw()?, attribute as u32) == 1)
        }
    }

    /// Returns `true` if value was deleted successfully
    pub fn delete(&mut self, key: &str) -> Result<bool, V8ValueError> {
        let inner = self.0.get()?;
        let key = CefString::new_raw(key);
        unsafe {
            let delete_value = (*inner).delete_value_bykey.unwrap();
            Ok(delete_value(inner, &key) == 1)
        }
    }
}

#[repr(u32)]
pub enum V8PropertyAttribute {
    None = chromium_sys::cef_v8_propertyattribute_t_V8_PROPERTY_ATTRIBUTE_NONE,
    ReadOnly = chromium_sys::cef_v8_propertyattribute_t_V8_PROPERTY_ATTRIBUTE_READONLY,
    DoNotEnum = chromium_sys::cef_v8_propertyattribute_t_V8_PROPERTY_ATTRIBUTE_DONTENUM,
    DoNotDelete = chromium_sys::cef_v8_propertyattribute_t_V8_PROPERTY_ATTRIBUTE_DONTDELETE,
}
