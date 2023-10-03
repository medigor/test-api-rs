use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse},
    Json,
};
use chrono::SecondsFormat;
use serde_json::json;

use crate::AppState;

pub async fn index() -> impl IntoResponse {
    Html(include_str!("index.html"))
}

pub async fn about(State(state): State<AppState>) -> impl IntoResponse {
    let build_date = env!("BUILD_DATE");
    Json(json!({
        "version": env!("CARGO_PKG_VERSION"),
        "build_date": build_date,
        "start_date": state.start_date.to_rfc3339_opts(SecondsFormat::Secs, true),
    }))
}

pub async fn counter() -> impl IntoResponse {
    static COUNTER: AtomicU64 = AtomicU64::new(1);

    Json(json!({"counter": COUNTER.fetch_add(1, Ordering::SeqCst)}))
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
    let json = serde_json::to_string(&map).unwrap();

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
        .map(|x| x.to_str().unwrap())
        .unwrap_or_default();

    Json(json!({"ip": ip}))
}
