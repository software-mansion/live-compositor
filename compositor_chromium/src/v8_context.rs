use log::warn;

use crate::{
    cef::V8Value,
    validated::{Validatable, Validated, ValidatedError},
};

/// JavaScript V8 engine context.
/// Available only on the renderer process
pub struct V8Context {
    inner: Validated<chromium_sys::cef_v8context_t>,
}

impl V8Context {
    pub(crate) fn new(v8_context: *mut chromium_sys::cef_v8context_t) -> Self {
        let inner = Validated(v8_context);
        Self { inner }
    }

    pub fn enter(&self) -> Result<V8ContextEntered<'_>, V8ContextError> {
        unsafe {
            let ctx = self.inner.get()?;
            let enter_context = (*ctx).enter.unwrap();
            enter_context(ctx);
        }

        Ok(V8ContextEntered(self))
    }

    pub fn global(&self) -> Result<V8Value, V8ContextError> {
        unsafe {
            let ctx = self.inner.get()?;
            let get_global = (*ctx).get_global.unwrap();
            Ok(V8Value::from_raw(get_global(ctx)))
        }
    }
}

pub struct V8ContextEntered<'a>(&'a V8Context);

impl<'a> Drop for V8ContextEntered<'a> {
    fn drop(&mut self) {
        unsafe {
            match self.0.inner.get() {
                Ok(ctx) => {
                    let exit_context = (*ctx).exit.unwrap();
                    exit_context(ctx);
                }
                Err(err) => warn!("Could not exit the context: {err}"),
            }
        }
    }
}

impl Validatable for chromium_sys::cef_v8context_t {
    fn is_valid(&mut self) -> bool {
        let is_valid = self.is_valid.unwrap();
        unsafe { is_valid(self) == 1 }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum V8ContextError {
    #[error("V8Context is no longer valid")]
    NotValid(#[from] ValidatedError),
}
