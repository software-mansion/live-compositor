use axum::{
    async_trait,
    extract::{rejection::JsonRejection, ws::WebSocketUpgrade, FromRequest, Request, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use compositor_pipeline::Pipeline;
use serde_json::{json, Value};

use crate::{
    error::ApiError,
    state::{ApiState, Response},
};

use self::{update_output::handle_output_update, ws::handle_ws_upgrade};

mod register_request;
mod unregister_request;
mod update_output;
mod ws;

pub fn routes(state: ApiState) -> Router {
    async fn handle_start(State(state): State<ApiState>) -> Result<Response, ApiError> {
        Pipeline::start(&state.pipeline);
        Ok(Response::Ok {})
    }

    Router::new()
        .route(
            "/api/input/:id/register",
            post(register_request::handle_input),
        )
        .route(
            "/api/output/:id/register",
            post(register_request::handle_output),
        )
        .route(
            "/api/renderer/register",
            post(register_request::handle_renderer),
        )
        .route(
            "/api/input/:id/unregister",
            post(unregister_request::handle_input),
        )
        .route(
            "/api/output/:id/unregister",
            post(unregister_request::handle_output),
        )
        .route(
            "/api/renderer/unregister",
            post(unregister_request::handle_renderer),
        )
        .route("/api/output/:id/update", post(handle_output_update))
        // Start request
        .route("/api/start", post(handle_start))
        // WebSocket - events
        .route("/ws", get(ws_handler))
        .route(
            "/status",
            get(axum::Json(json!({
                "instance_id": state.config.instance_id
            }))),
        )
        .with_state(state)
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    // finalize the upgrade process by returning upgrade callback.
    ws.on_upgrade(handle_ws_upgrade)
}

/// Wrap axum::Json to return serialization errors as json
pub(super) struct Json<T>(pub T);

#[async_trait]
impl<S, T> FromRequest<S> for Json<T>
where
    axum::Json<T>: FromRequest<S, Rejection = JsonRejection>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, axum::Json<Value>);

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let (parts, body) = req.into_parts();
        let req = Request::from_parts(parts, body);

        match axum::Json::<T>::from_request(req, state).await {
            Ok(value) => Ok(Self(value.0)),
            Err(rejection) => {
                let payload = json!({
                    "error_code": "MALFORMED_REQUEST",
                    "message": rejection.body_text(),
                });

                Err((rejection.status(), axum::Json(payload)))
            }
        }
    }
}
