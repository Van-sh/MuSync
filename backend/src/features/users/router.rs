use anyhow::Error;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use chrono::{Duration, Utc};
use sea_orm::DatabaseConnection;

use crate::{
    app::AppState,
    error::{ApiError, ApiResult},
    features::users::{self, AccessTokenClaims, LoginBody, RefreshTokenClaims, RegisterBody},
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
        .route("/login", post(login))
        .route("/logout", post(logout))
    // .route("/whoami", get(todo!()))
    // .route("/token", get(todo!()))
}

async fn register(
    State(db): State<DatabaseConnection>,
    jar: CookieJar,
    Json(RegisterBody {
        username,
        password,
        confirm_password,
    }): Json<RegisterBody>,
) -> ApiResult<impl IntoResponse> {
    if password != confirm_password {
        return Err(ApiError::BadRequest(
            "Password and Confirm Password must be same".into(),
        ));
    }

    let user = users::repo::find_user_by_username(&db, &username).await?;
    if user.is_some() {
        return Err(ApiError::Forbidden(format!("Username {username} is taken")));
    }

    let hashed_password = Argon2::default()
        .hash_password(password.as_bytes())
        .map_err(Error::new)?
        .to_string();

    let user = users::repo::create_user(&db, username, hashed_password).await?;

    sign_jwt(db, user, jar).await
}

async fn login(
    State(db): State<DatabaseConnection>,
    jar: CookieJar,
    Json(LoginBody { username, password }): Json<LoginBody>,
) -> ApiResult<impl IntoResponse> {
    let user = users::repo::find_user_by_username(&db, &username).await?;
    let Some(user) = user else {
        return Err(ApiError::NotFound(format!("User {username} doesn't exist")));
    };

    let verify = Argon2::default().verify_password(
        password.as_bytes(),
        &PasswordHash::new(&user.password).map_err(Error::new)?,
    );
    if verify.is_err() {
        return Err(ApiError::BadRequest("Invaldi username or password".into()));
    }

    sign_jwt(db, user, jar).await
}

async fn logout(
    State(db): State<DatabaseConnection>,
    jar: CookieJar,
) -> ApiResult<impl IntoResponse> {
    let Some(refresh_token) = jar.get(REFRESH_TOKEN_COOKIE_NAME) else {
        return Err(ApiError::Unauthorized);
    };
    let refresh_token = RefreshTokenClaims::decode_jwt(refresh_token.value().as_bytes())?;
    let _ = users::repo::delete_refresh_token(&db, refresh_token.jti).await;

    Ok((
        StatusCode::OK,
        jar.remove(REFRESH_TOKEN_COOKIE_NAME)
            .remove(ACCESS_TOKEN_COOKIE_NAME),
    ))
}

async fn sign_jwt(
    db: DatabaseConnection,
    user: entity::users::Model,
    jar: CookieJar,
) -> ApiResult<impl IntoResponse> {
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
