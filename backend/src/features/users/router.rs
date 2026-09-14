use axum::{
    Json, Router,
    response::IntoResponse,
    routing::{get, post},
};

use crate::{app::AppState, features::users::AuthBody};

pub fn router() -> Router<AppState> {
    Router::new()
        .without_v07_checks()
        .route("/register", post(register))
        .route("/login", post(todo!()))
        .route("/whoami", get(todo!()))
        .route("/token", get(todo!()))
}

fn register(Json(credentials): Json<AuthBody>) -> impl IntoResponse {
    todo!()
}
