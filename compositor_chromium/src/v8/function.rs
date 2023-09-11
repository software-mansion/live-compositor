use crate::validated::{Validated, ValidatedError};

use super::{
    value::{V8Value, V8ValueError},
    V8Context, V8ContextEntered, V8Exception, V8Object,
};

pub struct V8Function(pub(super) Validated<chromium_sys::cef_v8value_t>);

impl V8Function {
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

            let has_exception = (*result).has_exception.unwrap();
            if has_exception(result) == 1 {
                let get_exception = (*result).get_exception.unwrap();
                let exception = V8Exception::new(get_exception(result));
                return Err(V8FunctionError::CallException(exception));
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

#[derive(Debug, thiserror::Error)]
pub enum V8FunctionError {
    #[error("V8Function is no longer valid.")]
    FunctionNotValid(#[from] ValidatedError),

    #[error("V8Function arg at \"{1}\" is not valid.")]
    ArgNotValid(#[source] V8ValueError, usize),

    #[error("V8Function call throwed exception.")]
    CallException(#[source] V8Exception),
}
