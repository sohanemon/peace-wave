use axum::{Json, Router, routing::get};
use serde_json::json;
use tracing::{error, info};

use crate::config::Config;

pub fn router() -> Router {
    Router::new().route("/", get(health))
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({
        "name": "PeaceWave",
        "message": "PeaceWave API is running"
    }))
}

// NOTE: resolves after the first received signal; serve drains open connections and returns
async fn shutdown_signal() {
    let ctrl_c = async {
        match tokio::signal::ctrl_c().await {
            Ok(()) => info!("received SIGINT, shutting down"),
            Err(err) => error!(error = %err, "failed to listen for SIGINT"),
        }
    };

    #[cfg(unix)]
    let terminate = async {
        // INFO: SIGTERM is Unix-only; non-Unix builds block permanently on the fallback
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut sig) => {
                sig.recv().await;
                info!("received SIGTERM, shutting down");
            }
            Err(err) => error!(error = %err, "failed to listen for SIGTERM"),
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}

pub async fn serve(config: &Config) -> std::io::Result<()> {
    let listener = tokio::net::TcpListener::bind(config.bind_addr()).await?;

    info!(addr = %config.bind_addr(), "server listening");

    axum::serve(listener, router())
        .with_graceful_shutdown(shutdown_signal())
        .await
}
