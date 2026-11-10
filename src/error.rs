//! Centralized HTTP application errors and their JSON representation.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use tracing::error;

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    NotFound,
    Internal(String),
}

impl AppError {
    // NOTE: single mapping of variant to status, machine code, and client-safe message
    pub(crate) fn parts(self) -> (StatusCode, &'static str, String) {
        match self {
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, "bad_request", message),
            Self::NotFound => (
                StatusCode::NOT_FOUND,
                "not_found",
                "Resource not found".to_owned(),
            ),
            // INFO: details stay server-side; clients receive the generic message only
            Self::Internal(detail) => {
                error!(error = %detail, "internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "Internal server error".to_owned(),
                )
            }
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = self.parts();

        (status, Json(error_json(code, &message))).into_response()
    }
}

// NOTE: shared JSON envelope every error response must use
fn error_json(code: &str, message: &str) -> serde_json::Value {
    json!({ "error": { "code": code, "message": message } })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;

    async fn body_value(response: Response) -> serde_json::Value {
        let bytes = to_bytes(response.into_body(), 1024)
            .await
            .expect("body read");
        serde_json::from_slice(&bytes).expect("valid error JSON")
    }

    #[test]
    fn bad_request_maps_to_400() {
        let (status, code, _) = AppError::BadRequest("bad input".to_owned()).parts();

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(code, "bad_request");
    }

    #[test]
    fn not_found_maps_to_404_with_generic_message() {
        let (status, code, message) = AppError::NotFound.parts();

        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(code, "not_found");
        assert_eq!(message, "Resource not found");
    }

    #[test]
    fn internal_maps_to_500_and_returns_generic_message() {
        let (status, code, message) =
            AppError::Internal("postgres: connection refused".to_owned()).parts();

        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(code, "internal_error");
        assert_eq!(message, "Internal server error");
        assert!(!message.contains("postgres"));
    }

    #[test]
    fn error_json_has_nested_envelope() {
        let body = error_json("not_found", "Resource not found");

        assert_eq!(body["error"]["code"], "not_found");
        assert_eq!(body["error"]["message"], "Resource not found");
    }

    #[tokio::test]
    async fn into_response_serializes_json_envelope() {
        let response = AppError::NotFound.into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            body_value(response).await,
            error_json("not_found", "Resource not found"),
        );
    }

    #[tokio::test]
    async fn bad_request_detail_is_visible_to_client() {
        let response = AppError::BadRequest("missing id".to_owned()).into_response();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            body_value(response).await,
            error_json("bad_request", "missing id"),
        );
    }
}
