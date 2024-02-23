use tiny_http::Method;
use tracing::{debug, trace};

use crate::{
    api::{Api, ResponseHandler},
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
    let mut body: String = "".to_string();
    request
        .as_reader()
        .read_to_string(&mut body)
        .map_err(|err| ApiError::malformed_request(&err))?;
    trace!(?body, "Raw request: POST /--/api");

    let request = serde_json::from_str(&body).map_err(|err| ApiError::malformed_request(&err))?;
    debug!(?request, "Request: POST /--/api");

    api.handle_request(request)
}
