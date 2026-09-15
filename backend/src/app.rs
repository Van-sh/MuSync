use anyhow::Result;
use axum::{Router, extract::OriginalUri};
use axum_macros::FromRef;
use migration::sea_orm::DatabaseConnection;
use tower::ServiceBuilder;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};

use crate::{error::ApiError, features::users};

#[derive(Clone, FromRef)]
pub struct AppState {
    db: DatabaseConnection,
}

pub fn router(db: DatabaseConnection) -> Result<Router> {
    let state = AppState { db };

    Ok(Router::new()
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
        .with_state(state))
}
