use axum::Json;
use serde_json::json;

// NOTE: liveness signal only; downstream checks arrive with real dependencies
pub async fn handler() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}
