use crate::Status;
use axum_core::response::{IntoResponse, Response};
use http::{StatusCode, header};

/// JSON error response body
#[derive(serde::Serialize)]
struct ErrorResponse<'a> {
    error: &'a [Status],
}

static INTERNAL_ERROR_JSON: &str = r#"{"error":[{"code":500,"status":"INTERNAL","message":"Internal Server Error"}]}"#;

impl IntoResponse for Status {
    fn into_response(self) -> Response {
        let http_status = StatusCode::from(self.status);
        let body = ErrorResponse { error: &[self] };
        match serde_json::to_string(&body) {
            Ok(json) => (
                http_status,
                [(header::CONTENT_TYPE, "application/json")],
                json,
            )
                .into_response(),
            Err(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(header::CONTENT_TYPE, "application/json")],
                INTERNAL_ERROR_JSON,
            )
                .into_response(),
        }
    }
}
