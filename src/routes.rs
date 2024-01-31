use tiny_http::Method;

use crate::{
    api::{Api, Request, ResponseHandler},
    error::ApiError,
};

pub fn handle_request(
    api: &mut Api,
    request: &mut tiny_http::Request,
) -> Result<ResponseHandler, ApiError> {
    match (request.method(), request.url()) {
        (Method::Post, "/--/api") => handle_api_request(api, request),
        (Method::Get, "/status") => Ok(ResponseHandler::Ok),
        _ => Err(ApiError::new(
            "NOT FOUND",
            "Unknown endpoint".to_string(),
            tiny_http::StatusCode(404),
        )),
    }
}

fn handle_api_request(
    api: &mut Api,
    request: &mut tiny_http::Request,
) -> Result<ResponseHandler, ApiError> {
    let request = serde_json::from_reader::<_, Request>(request.as_reader())
        .map_err(|err| ApiError::malformed_request(&err))?;
    api.handle_request(request)
}
