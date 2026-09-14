use axum::Router;

use crate::app::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .without_v07_checks()
        .route("/register", todo!())
        .route("/login", todo!())
        .route("/whoami", todo!())
        .route("/token", todo!())
}
