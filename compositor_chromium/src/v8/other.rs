use crate::v8::V8Value;
use crate::validated::Validated;

pub struct V8GenericValue(pub(super) Validated<chromium_sys::cef_v8value_t>);

pub struct V8Undefined(pub(super) Validated<chromium_sys::cef_v8value_t>);

impl Default for V8Undefined {
    fn default() -> Self {
        Self::new()
    }
}

impl V8Undefined {
    pub fn new() -> Self {
        let inner = unsafe { chromium_sys::cef_v8value_create_undefined() };
        Self(Validated::new(inner))
    }
}

impl From<V8Undefined> for V8Value {
    fn from(value: V8Undefined) -> Self {
        Self::Undefined(value)
    }
}

pub struct V8Null(pub(super) Validated<chromium_sys::cef_v8value_t>);

impl Default for V8Null {
    fn default() -> Self {
        Self::new()
    }
}

impl V8Null {
    pub fn new() -> Self {
        let inner = unsafe { chromium_sys::cef_v8value_create_null() };
        Self(Validated::new(inner))
    }
}

impl From<V8Null> for V8Value {
    fn from(value: V8Null) -> Self {
        Self::Null(value)
    }
}
