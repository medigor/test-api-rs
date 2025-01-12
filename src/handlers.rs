use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse},
    Json,
};
use futures::{SinkExt, StreamExt};
use serde::Serialize;
use tokio::time::{timeout, Instant};

use crate::AppState;

pub async fn index() -> impl IntoResponse {
    Html(include_str!("index.html"))
}

pub async fn about(State(state): State<AppState>) -> impl IntoResponse {
    #[derive(Serialize)]
    struct Response {
        version: &'static str,
        build_date: &'static str,
        start_date: &'static str,
    }
    Json(Response {
        version: env!("CARGO_PKG_VERSION"),
        build_date: env!("BUILD_DATE"),
        start_date: state.start_date,
    })
}

pub async fn counter() -> impl IntoResponse {
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    #[derive(Serialize)]
    struct Response {
        counter: u64,
    }
    Json(Response {
        counter: COUNTER.fetch_add(1, Ordering::SeqCst),
    })
}

pub async fn headers(headers: HeaderMap) -> impl IntoResponse {
    let nginx_headers = [
        "x-forwarded-for",
        "x-forwarded-host",
        "x-forwarded-port",
        "x-forwarded-proto",
        "x-forwarded-scheme",
        "x-real-ip",
        "x-request-id",
        "x-scheme",
    ];
    let map = headers
        .iter()
        .filter(|(h, _)| !nginx_headers.contains(&h.as_str()))
        .map(|(name, value)| (name.as_str(), value.to_str().unwrap_or("invalid string")))
        .collect::<BTreeMap<_, _>>();
    let json = serde_json::to_string(&map).unwrap_or_default();

    ([(header::CONTENT_TYPE, "application/json")], json)
}

pub async fn sleep(Path(duration): Path<u64>) -> impl IntoResponse {
    if duration > 30_000 {
        return StatusCode::BAD_REQUEST;
    }
    tokio::time::sleep(Duration::from_millis(duration)).await;
    StatusCode::OK
}

pub async fn ip(headers: HeaderMap) -> impl IntoResponse {
    let ip = headers
        .get("x-real-ip")
        .map(|x| x.to_str().unwrap_or_default())
        .unwrap_or_default()
        .to_owned();
    #[derive(Serialize)]
    struct Response {
        ip: String,
    }
    Json(Response { ip })
}

pub async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.max_message_size(1024).on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    let expires = Instant::now()
        .checked_add(Duration::from_secs(10))
        .expect("Failed add duration");
    if socket
        .send(Message::Text("Hello from Rust!".into()))
        .await
        .is_err()
    {
        return;
    }

    loop {
        let result = timeout(expires - Instant::now(), socket.next()).await;
        let Ok(msg) = result else {
            let _ = socket.send(Message::text("Timeout expired")).await;
            // timeout
            break;
        };

        let Some(Ok(msg)) = msg else {
            break;
        };

        match msg {
            Message::Text(str) => {
                if socket.send(Message::Text(str)).await.is_err() {
                    break;
                }
            }
            Message::Binary(vec) => {
                if socket.send(Message::Binary(vec)).await.is_err() {
                    break;
                }
            }
            Message::Ping(_) => (),
            Message::Pong(_) => (),
            Message::Close(_) => break,
        }
    }
    if socket.send("Bye Bye".into()).await.is_ok() {
        let _ = socket.close().await;
    }
}
