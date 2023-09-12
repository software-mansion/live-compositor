use std::{fmt::Display, os::raw::c_int};

use crate::{
    cef_ref::{CefRefData, CefStruct},
    cef_string::CefString,
    validated::{Validated, ValidatedError},
};

use super::{
    value::{V8Value, V8ValueError},
    V8ContextEntered, V8Exception, V8Object,
};

pub struct V8Function(pub(super) Validated<chromium_sys::cef_v8value_t>);

impl V8Function {
    pub fn new<F>(name: &str, func: F) -> Self
    where
        F: Fn(&[V8Value]) -> Result<V8Value, NativeFunctionError>,
    {
        let name = CefString::new_raw(name);
        let handler = CefRefData::new_ptr(V8Handler(func));
        let inner = unsafe { chromium_sys::cef_v8value_create_function(&name, handler) };

        Self(Validated::new(inner))
    }

    pub fn call(
        &self,
        args: &[V8Value],
        ctx_entered: &V8ContextEntered,
    ) -> Result<V8Value, V8FunctionError> {
        self.inner_call(None, args, ctx_entered)
    }

    pub(super) fn call_as_method(
        &self,
        this: &V8Object,
        args: &[V8Value],
        ctx_entered: &V8ContextEntered,
    ) -> Result<V8Value, V8FunctionError> {
        self.inner_call(Some(this), args, ctx_entered)
    }

    fn inner_call(
        &self,
        this: Option<&V8Object>,
        args: &[V8Value],
        _ctx_entered: &V8ContextEntered,
    ) -> Result<V8Value, V8FunctionError> {
        let inner = self.0.get()?;

        let this = match this {
            Some(this) => this.0.get()?,
            None => std::ptr::null_mut(),
        };
        let args = args
            .iter()
            .enumerate()
            .map(|(i, v)| {
                v.get_raw()
                    .map_err(|err| V8FunctionError::ArgNotValid(err, i))
            })
            .collect::<Result<Vec<_>, _>>()?;

        unsafe {
            let execute = (*inner).execute_function.unwrap();
            let result = execute(inner, this, args.len(), args.as_ptr());

            // Some functions do not give us access to the exception. They just print it and return null
            if result.is_null() {
                return Err(V8FunctionError::CallException(None));
            }
            let has_exception = (*result).has_exception.unwrap();
            if has_exception(result) == 1 {
                let get_exception = (*result).get_exception.unwrap();
                let exception = V8Exception::new(get_exception(result));
                return Err(V8FunctionError::CallException(Some(exception)));
            }
            Ok(V8Value::from_raw(result))
        }
    }
}

impl From<V8Function> for V8Value {
    fn from(value: V8Function) -> Self {
        Self::Function(value)
    }
}

struct V8Handler<F: Fn(&[V8Value]) -> Result<V8Value, NativeFunctionError>>(F);

impl<F: Fn(&[V8Value]) -> Result<V8Value, NativeFunctionError>> CefStruct for V8Handler<F> {
    type CefType = chromium_sys::cef_v8handler_t;

    fn cef_data(&self) -> Self::CefType {
        chromium_sys::cef_v8handler_t {
            base: unsafe { std::mem::zeroed() },
            execute: Some(Self::execute),
        }
    }

    fn base_mut(cef_data: &mut Self::CefType) -> &mut chromium_sys::cef_base_ref_counted_t {
        &mut cef_data.base
    }
}

impl<F: Fn(&[V8Value]) -> Result<V8Value, NativeFunctionError>> V8Handler<F> {
    extern "C" fn execute(
        self_: *mut chromium_sys::cef_v8handler_t,
        _name: *const chromium_sys::cef_string_t,
        _object: *mut chromium_sys::cef_v8value_t,
        arguments_count: usize,
        arguments: *const *mut chromium_sys::cef_v8value_t,
        retval: *mut *mut chromium_sys::cef_v8value_t,
        exception: *mut chromium_sys::cef_string_t,
    ) -> c_int {
        const HANDLED: c_int = 1;

        unsafe {
            let self_ref = CefRefData::<Self>::from_cef(self_);
            let args: Vec<V8Value> = std::slice::from_raw_parts(arguments, arguments_count)
                .iter()
                .cloned()
                .map(V8Value::from_raw)
                .collect();

            let result = self_ref.0(&args)
                .map_err(V8FunctionError::NativeFuncError)
                .and_then(|v| Ok(v.get_raw()?));
            match result {
                Ok(v) => {
                    *retval = v;
                }
                Err(err) => {
                    let err_msg = CefString::new_raw(err.to_string());
                    *exception = err_msg;
                    return HANDLED;
                }
            }
        }

        HANDLED
    }
}

#[derive(Debug, thiserror::Error)]
pub enum V8FunctionError {
    #[error("V8Function is no longer valid.")]
    FunctionNotValid(#[from] ValidatedError),

    #[error("V8Function arg at \"{1}\" is not valid.")]
    ArgNotValid(#[source] V8ValueError, usize),

    #[error("V8Function call throwed exception.")]
    CallException(#[source] Option<V8Exception>),

    #[error(transparent)]
    V8ValueError(#[from] V8ValueError),

    #[error("Native function failed: {0}")]
    NativeFuncError(#[from] NativeFunctionError),
}

#[derive(Debug)]
pub struct NativeFunctionError(pub String);

impl std::error::Error for NativeFunctionError {}

impl Display for NativeFunctionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
