use axum::{http::StatusCode, response::IntoResponse};
use axum_extra::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Not Found: {0}")]
    NotFound(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ApiError::NotFound(message) => {
                (StatusCode::NOT_FOUND, json!({"message": message})).into_response()
            }
        }
    }
}
