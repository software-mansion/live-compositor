use crate::{
    cef_string::CefString,
    validated::{Validated, ValidatedError},
};

use super::{value::V8Value, V8ContextEntered, V8ValueError};

pub struct V8Object(pub(super) Validated<chromium_sys::cef_v8value_t>);

impl V8Object {
    pub fn has(&self, key: &str) -> Result<bool, V8ObjectError> {
        let inner = self.0.get()?;
        let key = CefString::new_raw(key);
        unsafe {
            let has_value = (*inner).has_value_bykey.unwrap();
            Ok(has_value(inner, &key) == 1)
        }
    }

    pub fn get(&self, key: &str) -> Result<V8Value, V8ObjectError> {
        let inner = self.0.get()?;
        let key = CefString::new_raw(key);
        unsafe {
            let get_value = (*inner).get_value_bykey.unwrap();
            let value = get_value(inner, &key);
            if value.is_null() {
                return Err(V8ObjectError::FieldNotFound);
            }

            Ok(V8Value::from_raw(value))
        }
    }

    pub fn set(
        &mut self,
        key: &str,
        value: &V8Value,
        attribute: V8PropertyAttribute,
        _context_entered: &V8ContextEntered,
    ) -> Result<(), V8ObjectError> {
        let inner = self.0.get()?;
        let key = CefString::new_raw(key);
        unsafe {
            let set_value = (*inner).set_value_bykey.unwrap();
            if set_value(inner, &key, value.get_raw()?, attribute as u32) != 1 {
                return Err(V8ObjectError::SetFailed);
            }
            Ok(())
        }
    }

    pub fn delete(
        &mut self,
        key: &str,
        _context_entered: &V8ContextEntered,
    ) -> Result<(), V8ObjectError> {
        let inner = self.0.get()?;
        let key = CefString::new_raw(key);
        unsafe {
            let delete_value = (*inner).delete_value_bykey.unwrap();
            if delete_value(inner, &key) != 1 {
                return Err(V8ObjectError::DeleteFailed);
            }

            Ok(())
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

#[derive(Debug, thiserror::Error)]
pub enum V8ObjectError {
    #[error("V8Object is no longer valid")]
    ObjectNotValid(#[from] ValidatedError),

    #[error(transparent)]
    V8ValueError(#[from] V8ValueError),

    #[error("V8Object field not found")]
    FieldNotFound,

    #[error("Failed to set V8Object field")]
    SetFailed,

    #[error("Failed to delete V8Object field")]
    DeleteFailed,
}
