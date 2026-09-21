use std::sync::LazyLock;

use anyhow::Context;

#[derive(Debug)]
pub struct Env {
    pub port: u16,
    pub db: String,
    pub jwt_secret: String,
}

impl Env {
    pub fn get() -> &'static LazyLock<Self> {
        &INSTANCE
    }
}

static INSTANCE: LazyLock<Env> = LazyLock::new(|| {
    dotenvy::dotenv()
        .map_err(|_| tracing::debug!(".env not found"))
        .ok();

    let port: u16 = std::env::var("PORT")
        .unwrap_or_default()
        .parse::<u16>()
        .unwrap_or(3435);

    let db = std::env::var("DATABASE_URL")
        .context("DATABASE_URL env var not found")
        .unwrap();

    let jwt_secret = std::env::var("JWT_SECRET")
        .context("JWT_SECRET env var not found")
        .unwrap();

    Env {
        port,
        db,
        jwt_secret,
    }
});
