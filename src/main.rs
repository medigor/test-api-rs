use axum::{
    extract::Path,
    http::{header, HeaderMap, Method, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use std::{
    collections::HashMap,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};
use tokio::signal;
use tokio::time::sleep;
use tower_http::cors::{Any, CorsLayer};

#[derive(Serialize)]
struct CounterResponse {
    counter: u64,
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("signal received, starting graceful shutdown");
}

async fn headers(headers: HeaderMap) -> impl IntoResponse {
    let mut map = HashMap::<&str, &str>::new();
    for (name, value) in headers.iter() {
        map.insert(name.as_str(), value.to_str().unwrap_or("invalid string"));
    }
    let json = serde_json::to_string(&map).unwrap();

    ([(header::CONTENT_TYPE, "application/json")], json)
}

#[tokio::main]
async fn main() {
    let counter = Box::leak(Box::new(AtomicU64::new(1)));

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any);

    let app = Router::new()
        .route(
            "/counter/json",
            post(|| async {
                Json(CounterResponse {
                    counter: counter.fetch_add(1, Ordering::SeqCst),
                })
            }),
        )
        .route(
            "/counter",
            post(|| async { format!("{}", counter.fetch_add(1, Ordering::SeqCst)) }),
        )
        .route(
            "/sleep/:duration",
            post(|Path(duration): Path<u64>| async move {
                if duration > 30_000 {
                    return StatusCode::BAD_REQUEST;
                }
                sleep(Duration::from_millis(duration)).await;
                StatusCode::OK
            }),
        )
        .route("/headers", get(headers))
        .route("/headers", post(headers))
        .layer(cors);

    axum::Server::bind(&([0, 0, 0, 0], 8080).into())
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}
