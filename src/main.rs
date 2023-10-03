mod handlers;

use axum::{
    http::Method,
    routing::{get, post},
    Router,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use tokio::signal;

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

#[derive(Clone)]
pub struct AppState {
    start_date: DateTime<Utc>,
}

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any);

    let state = AppState {
        start_date: Utc::now(),
    };

    let app = Router::new()
        .route("/", get(handlers::index))
        .route("/about", get(handlers::about))
        .route("/counter", post(handlers::counter))
        .route("/sleep/:duration", post(handlers::sleep))
        .route("/headers", get(handlers::headers))
        .route("/headers", post(handlers::headers))
        .route("/ip", get(handlers::ip))
        .layer(cors)
        .with_state(state);

    axum::Server::bind(&([0, 0, 0, 0], 8080).into())
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}
