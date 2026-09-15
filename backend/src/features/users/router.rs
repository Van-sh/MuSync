use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use axum_extra::json;
use sea_orm::DatabaseConnection;

use crate::{
    app::AppState,
    error::ApiError,
    features::users::{self, AuthBody},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .without_v07_checks()
        .route("/register", post(register))
    // .route("/login", post(todo!()))
    // .route("/whoami", get(todo!()))
    // .route("/token", get(todo!()))
}

async fn register(
    State(db): State<DatabaseConnection>,
    Json(AuthBody { username, password }): Json<AuthBody>,
) -> Result<impl IntoResponse, ApiError> {
    let user = users::repo::find_user_by_username(&db, &username).await?;

    Ok((StatusCode::OK, json!({"test": "test"})))
}
