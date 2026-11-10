pub mod health;

use axum::{Router, routing::get};

// NOTE: /v1 is the versioned public API surface
pub fn api_router() -> Router {
    Router::new().route("/health", get(health::handler))
}
