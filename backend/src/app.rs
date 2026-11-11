use std::sync::Arc;

use axum::{Router, extract::OriginalUri, response::IntoResponse, routing::get};
use axum_extra::json;
use axum_macros::FromRef;
use better_auth::{BetterAuth, integrations::axum::AxumIntegration};
use migration::sea_orm::DatabaseConnection;
use tokio::sync::Mutex;
use tower::ServiceBuilder;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};

use crate::{
    error::ApiError,
    features::{
        auth::AppAuthSchema,
        rooms::{manager::RoomManager, router::RoomsRouter},
    },
};

#[derive(Clone, FromRef)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub auth: Arc<BetterAuth<AppAuthSchema>>,
    pub room_manager: Arc<Mutex<RoomManager>>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .without_v07_checks()
        .route("/health", get(health))
        .nest(
            "/api/v1",
            Router::new()
                .nest("/auth", state.auth.clone().axum_router_with_state())
                .nest("/rooms", RoomsRouter::get()),
        )
        .fallback(async |OriginalUri(original_uri): OriginalUri| {
            ApiError::NotFound(format!("Can't find \"{original_uri}\" on this server!"))
        })
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::very_permissive())
                .layer(CompressionLayer::new()),
        )
        .with_state(state)
}

async fn health() -> impl IntoResponse {
    json!({"health": "working"})
}
