use std::time::Duration;

use axum::{
    Json, Router,
    body::Body,
    http::{Request, Response},
    routing::get,
};
use serde_json::json;
use tower_http::trace::TraceLayer;
use tracing::{Span, debug_span, error, info, warn};

use crate::config::Config;
use crate::error::AppError;
use crate::routes;

pub fn router() -> Router {
    Router::new()
        .route("/", get(health))
        .nest("/v1", routes::api_router())
        .fallback(not_found)
        // NOTE: covers every route added above, including fallback and nested /v1
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<Body>| {
                    // SECURITY: only non-sensitive request metadata enters span fields
                    debug_span!(
                        "http request",
                        method = %request.method(),
                        uri = %request.uri(),
                        version = ?request.version(),
                    )
                })
                .on_request(|request: &Request<Body>, _span: &Span| {
                    // SECURITY: only non-sensitive request metadata is logged
                    info!(
                        method = %request.method(),
                        uri = %request.uri(),
                        version = ?request.version(),
                        "request started",
                    );
                })
                .on_response(
                    |response: &Response<Body>, latency: Duration, _span: &Span| {
                        let status = response.status().as_u16();
                        let latency_ms = latency.as_millis();

                        if response.status().is_server_error() {
                            error!(status, latency_ms, "request finished");
                        } else if response.status().is_client_error() {
                            warn!(status, latency_ms, "request finished");
                        } else {
                            info!(status, latency_ms, "request finished");
                        }
                    },
                ),
        )
}

// NOTE: JSON-shaped 404 via AppError; axum's default fallback body is empty
async fn not_found() -> AppError {
    AppError::NotFound
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

#[cfg(test)]
mod tests {
    use super::router;

    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use serde_json::Value;
    use serde_json::json;
    use tower::ServiceExt;

    async fn call(uri: &str) -> (StatusCode, Value) {
        let response = router()
            .oneshot(
                Request::builder()
                    .uri(uri)
                    .body(Body::empty())
                    .expect("valid request"),
            )
            .await
            .expect("response produced");

        let status = response.status();
        let bytes = to_bytes(response.into_body(), 1024)
            .await
            .expect("body read");
        let value = serde_json::from_slice(&bytes).expect("valid JSON body");
        (status, value)
    }

    #[tokio::test]
    async fn root_keeps_original_response() {
        let (status, body) = call("/").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            body,
            json!({ "name": "PeaceWave", "message": "PeaceWave API is running" }),
        );
    }

    #[tokio::test]
    async fn versioned_health_responds_200() {
        let (status, _) = call("/v1/health").await;

        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn versioned_health_body_is_status_ok() {
        let (_, body) = call("/v1/health").await;

        assert_eq!(body, json!({ "status": "ok" }));
    }

    #[tokio::test]
    async fn unknown_route_returns_404_envelope() {
        let (status, body) = call("/unknown").await;

        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(
            body,
            json!({ "error": { "code": "not_found", "message": "Resource not found" } }),
        );
    }
}
