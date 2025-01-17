mod handlers;

use std::error;

use axum::{
    http::Method,
    routing::{any, get, post},
    Router,
};
use chrono::{SecondsFormat, Utc};
use tokio::{net::TcpListener, signal};

use tower_http::cors::{Any, CorsLayer};

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
    start_date: &'static str,
}

fn main() -> Result<(), Box<dyn error::Error>> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(run())
}
async fn run() -> Result<(), Box<dyn error::Error>> {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any);

    let state = AppState {
        start_date: Box::leak(
            Utc::now()
                .to_rfc3339_opts(SecondsFormat::Secs, true)
                .into_boxed_str(),
        ),
    };

    let app = Router::new()
        .route("/", get(handlers::index))
        .route("/about", get(handlers::about))
        .route("/counter", post(handlers::counter))
        .route("/sleep/{duration}", post(handlers::sleep))
        .route("/headers", get(handlers::headers))
        .route("/headers", post(handlers::headers))
        .route("/ip", get(handlers::ip))
        .route("/ws", any(handlers::ws_handler))
        .layer(cors)
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:8080").await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}
