/// Holds pointer to validatable data and checks if it's valid on retrieval
pub(crate) struct Validated<T: Validatable>(pub *mut T);

impl<T: Validatable> Validated<T> {
    pub fn get(&self) -> Result<*mut T, ValidatedError> {
        if self.0.is_null() {
            return Err(ValidatedError::NotValid);
        }

        unsafe {
            if !(*self.0).is_valid() {
                return Err(ValidatedError::NotValid);
            }
        }

        Ok(self.0)
    }
}
pub(crate) trait Validatable {
    /// Should return `true` if data is alive and can be used.
    /// Most CEF structs have `is_valid` method which can be used for implementing this
    fn is_valid(&mut self) -> bool;
}

#[derive(Debug, thiserror::Error)]
pub enum ValidatedError {
    #[error("Data is no longer valid")]
    NotValid,
}
