use axum::{
    async_trait,
    extract::{rejection::JsonRejection, ws::WebSocketUpgrade, FromRequest, Request, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use tracing::debug;

use crate::api::{self, Api};

use self::ws::handle_ws_upgrade;

mod ws;

pub fn routes(api: Api) -> Router {
    Router::new()
        .route("/--/api", post(handle_api_request))
        .route("/--/ws", get(ws_handler))
        .route(
            "/status",
            get(axum::Json(json!({
                "instance_id": api.config.instance_id
            }))),
        )
        .with_state(api)
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    // finalize the upgrade process by returning upgrade callback.
    ws.on_upgrade(handle_ws_upgrade)
}

async fn handle_api_request(
    State(api): State<Api>,
    Json(request): Json<api::Request>,
) -> impl IntoResponse {
    debug!(?request, "Received API request");
    api.handle_request(request).await
}

/// Wrap axum::Json to return serialization errors as json
struct Json<T>(pub T);

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
