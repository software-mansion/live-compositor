use std::time::Duration;

use axum::http::HeaderValue;
use rand::{rngs::StdRng, Rng, SeedableRng};
use tokio::time::sleep;
use tracing::error;

use crate::pipeline::whip_whep::error::WhipServerError;

pub async fn validate_token(
    expected_token: Option<String>,
    auth_header_value: Option<&HeaderValue>,
) -> Result<(), WhipServerError> {
    match (expected_token, auth_header_value) {
        (Some(bearer_token), Some(auth_str)) => {
            let auth_str = auth_str.to_str().map_err(|_| {
                WhipServerError::Unauthorized("Invalid UTF-8 in header".to_string())
            })?;

            if let Some(token_from_header) = auth_str.strip_prefix("Bearer ") {
                if token_from_header == bearer_token {
                    Ok(())
                } else {
                    let mut rng = StdRng::from_entropy();
                    let millis = rng.gen_range(50..1000);
                    sleep(Duration::from_millis(millis)).await;
                    error!("Invalid or mismatched token provided");
                    Err(WhipServerError::Unauthorized(
                        "Invalid or mismatched token provided".to_string(),
                    ))
                }
            } else {
                Err(WhipServerError::Unauthorized(
                    "Authorization header format incorrect".to_string(),
                ))
            }
        }
        _ => Err(WhipServerError::Unauthorized(
            "Expected token and authorization header required".to_string(),
        )),
    }
}
