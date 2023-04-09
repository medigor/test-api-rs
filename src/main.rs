use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use axum::{extract::Path, routing::post, Json, Router};
use serde::Serialize;
use tokio::time::sleep;

#[derive(Serialize)]
struct CounterResponse {
    counter: u64,
}

#[tokio::main]
async fn main() {
    let counter = Box::leak(Box::new(AtomicU64::new(1)));

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
                sleep(Duration::from_millis(duration)).await;
                "Ok"
            }),
        );

    axum::Server::bind(&([0, 0, 0, 0], 8080).into())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
