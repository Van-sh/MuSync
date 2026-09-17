use axum::{http::StatusCode, response::IntoResponse};
use axum_extra::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Forbidden")]
    Forbidden(String),
    #[error("Not Found: {0}")]
    NotFound(String),

    #[error("Internal Server Error: {0}")]
    InternalError(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ApiError::Forbidden(message) => (StatusCode::FORBIDDEN, json!({"message": message})),
            ApiError::NotFound(message) => (StatusCode::NOT_FOUND, json!({"message": message})),
            ApiError::InternalError(report) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"message": format!("Something went wrong: {report}" )}),
            ),
        }
        .into_response()
    }
}
