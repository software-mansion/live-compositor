use axum::{
    async_trait,
    extract::{rejection::JsonRejection, ws::WebSocketUpgrade, FromRequest, Request, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
    Router,
};
use compositor_pipeline::Pipeline;
use serde_json::{json, Value};

use crate::{
    api::{Api, Response},
    error::ApiError,
};

use self::{update_output::handle_output_update, ws::handle_ws_upgrade};

mod register_request;
mod unregister_request;
mod update_output;
mod ws;

pub fn routes(api: Api) -> Router {
    let rtp_input = Router::new()
        .route("/", put(register_request::handle_rtp_input_stream))
        .route(
            "/:id/unregister",
            post(unregister_request::handle_rtp_input_stream),
        );

    let mp4_input = Router::new().route("/mp4", put(register_request::handle_mp4));

    let output = Router::new()
        .route(
            "/rtp-stream",
            put(register_request::handle_rtp_output_stream),
        )
        .route(
            "/rtp-stream/:id/unregister",
            post(unregister_request::handle_rtp_output_stream),
        )
        .route("/rtp-stream/:id", post(handle_output_update));

    let web_renderer = Router::new()
        .route("/", put(register_request::handle_web_renderer))
        .route(
            "/:id/unregister",
            post(unregister_request::handle_web_renderer),
        );

    let image_renderer = Router::new()
        .route("/", put(register_request::handle_image))
        .route("/:id/unregister", post(unregister_request::handle_image));

    let shader_renderer = Router::new()
        .route("/", put(register_request::handle_shader))
        .route("/:id/unregister", post(unregister_request::handle_shader));

    async fn handle_start(State(api): State<Api>) -> Result<Response, ApiError> {
        Pipeline::start(&api.pipeline);
        Ok(Response::Ok {})
    }

    Router::new()
        .nest("/--/api/input/rtp-stream", rtp_input)
        .nest("/--/api/input/mp4", mp4_input)
        .nest("/--/api/output", output)
        .nest("/--/api/web-renderer", web_renderer)
        .nest("/--/api/image", image_renderer)
        .nest("/--/api/shader", shader_renderer)
        // Start request
        .route("/--/api/start", post(handle_start))
        // WebSocket - events
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
