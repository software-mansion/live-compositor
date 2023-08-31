use compositor_pipeline::error::{ErrorType, PipelineError};
use tiny_http::StatusCode;

pub struct ApiError {
    pub error_code: &'static str,
    pub message: String,
    pub http_status_code: tiny_http::StatusCode,
}

impl<T: Into<PipelineError>> From<T> for ApiError {
    fn from(err: T) -> Self {
        let err: PipelineError = err.into();
        ApiError {
            error_code: err.error_code,
            message: err.message,
            http_status_code: match err.error_type {
                ErrorType::UserError => StatusCode(400),
                ErrorType::ServerError => StatusCode(500),
                ErrorType::EntityNotFound => StatusCode(404),
            },
        }
    }
}
