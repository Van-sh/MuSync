use axum::{Router, extract::OriginalUri};
use tower::ServiceBuilder;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};

use crate::{error::ApiError, features::users};

#[derive(Clone)]
pub struct AppState;

pub fn router() -> Router {
    Router::new()
        .without_v07_checks()
        .nest("/auth", users::router())
        .fallback(async |OriginalUri(original_uri): OriginalUri| {
            ApiError::NotFound(format!("Can't find {original_uri} on this server!"))
        })
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::very_permissive())
                .layer(CompressionLayer::new()),
        )
        .with_state(AppState)
}
