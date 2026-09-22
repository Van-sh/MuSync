use anyhow::{Error, Result};
use argon2::{Argon2, PasswordHasher};
use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use chrono::{Duration, Utc};
use sea_orm::DatabaseConnection;

use crate::{
    app::AppState,
    error::ApiError,
    features::users::{self, AccessTokenClaims, AuthBody, RefreshTokenClaims},
    prelude::ToTimeDuration,
};

const REFRESH_TOKEN_COOKIE_NAME: &str = "musync_refresh_token";
const REFRESH_TOKEN_DURATION: Duration = Duration::weeks(1);
const ACCESS_TOKEN_COOKIE_NAME: &str = "musync_access_token";
const ACCESS_TOKEN_DURATION: Duration = Duration::minutes(5);

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

    let now = Utc::now().naive_utc();

    let refresh_token =
        users::repo::create_refresh_token(&db, user.id, now + REFRESH_TOKEN_DURATION).await?;
    let refresh_token = RefreshTokenClaims::from(refresh_token).encode_jwt()?;
    let refresh_token = Cookie::build((REFRESH_TOKEN_COOKIE_NAME, refresh_token))
        .http_only(true)
        .max_age(REFRESH_TOKEN_DURATION.to_time_duration());

    let access_token = AccessTokenClaims {
        sub: user.id,
        username: user.username,
        exp: now + ACCESS_TOKEN_DURATION,
    }
    .encode_jwt()?;
    let access_token = Cookie::build((ACCESS_TOKEN_COOKIE_NAME, access_token))
        .http_only(true)
        .max_age(ACCESS_TOKEN_DURATION.to_time_duration());

    Ok((StatusCode::OK, jar.add(refresh_token).add(access_token)))
}
