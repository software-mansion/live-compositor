use std::thread;

use axum::{
    async_trait,
    extract::{
        rejection::JsonRejection,
        ws::{Message, WebSocket, WebSocketUpgrade},
        FromRequest, Request, State,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use compositor_render::event_handler::{subscribe, Event};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::sync::mpsc::channel;
use tracing::debug;

use crate::{
    api::{self, Api},
    config::config,
};

pub fn routes(api: Api) -> Router {
    Router::new()
        .route("/--/api", post(handle_api_request))
        .route("/--/ws", get(ws_handler))
        .route(
            "/status",
            get(axum::Json(json!({
                "instance_id": config().instance_id
            }))),
        )
        .with_state(api)
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    // finalize the upgrade process by returning upgrade callback.
    ws.on_upgrade(handle_socket)
}

async fn handle_api_request(
    State(api): State<Api>,
    Json(request): Json<api::Request>,
) -> impl IntoResponse {
    debug!(?request, "Received API request");
    api.handle_request(request)
}

async fn handle_socket(socket: WebSocket) {
    enum InternalMessage {
        Event(Event),
        Close,
        Pong(Vec<u8>),
    }
    let (mut socket_sender, mut socket_receiver) = socket.split();
    let (event_sender, mut event_receiver) = channel(100);

    let event_sender_2 = event_sender.clone();
    thread::Builder::new()
        .name("Web socket thread".to_string())
        .spawn(move || {
            let receiver = subscribe();
            for event in receiver {
                if event_sender_2
                    .blocking_send(InternalMessage::Event(event))
                    .is_err()
                {
                    return;
                }
            }
        })
        .unwrap();

    tokio::spawn(async move {
        while let Some(event) = event_receiver.recv().await {
            match event {
                InternalMessage::Event(event) => {
                    let serialized = event_to_json(event).to_string();
                    if let Err(err) = socket_sender.send(Message::Text(serialized)).await {
                        debug!(%err, "WebSocket send error.");
                        return;
                    }
                }
                InternalMessage::Close => {
                    if let Err(err) = socket_sender.send(Message::Close(None)).await {
                        debug!(%err, "WebSocket send error.");
                        return;
                    }
                }
                InternalMessage::Pong(data) => {
                    if let Err(err) = socket_sender.send(Message::Pong(data)).await {
                        debug!(%err, "WebSocket send error.");
                        return;
                    }
                }
            }
        }
    });

    tokio::spawn(async move {
        while let Some(Ok(msg)) = socket_receiver.next().await {
            match msg {
                Message::Close(_) => {
                    let _ = event_sender.send(InternalMessage::Close).await;
                    return;
                }
                Message::Ping(data) => {
                    let _ = event_sender.send(InternalMessage::Pong(data)).await;
                }
                msg => {
                    debug!(?msg, "Received ws message.")
                }
            }
        }
    });
}

fn event_to_json(event: Event) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert("type".to_string(), event.kind.into());
    for (key, value) in event.properties {
        map.insert(key, value.into());
    }
    map.into()
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
