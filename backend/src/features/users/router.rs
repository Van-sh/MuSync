use anyhow::{Error, Result};
use argon2::{Argon2, PasswordHasher};
use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use axum_extra::{extract::CookieJar, json};
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
    // .route("/logout", post(todo!()))
    // .route("/whoami", get(todo!()))
    // .route("/token", get(todo!()))
}

async fn register(
    State(db): State<DatabaseConnection>,
    jar: CookieJar,
    Json(AuthBody { username, password }): Json<AuthBody>,
) -> Result<impl IntoResponse, ApiError> {
    let user = users::repo::find_user_by_username(&db, &username).await?;
    if user.is_some() {
        return Err(ApiError::Forbidden(format!("Username {username} is taken")));
    }

    let argon2 = Argon2::default();
    let hashed_password = argon2
        .hash_password(password.as_bytes())
        .map_err(Error::new)?
        .to_string();

    let user = users::repo::create_user(&db, username, hashed_password).await?;

    Ok((StatusCode::OK, json!({"test": "test"})))
}
