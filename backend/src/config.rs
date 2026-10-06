use std::sync::LazyLock;

use anyhow::Context;

#[derive(Debug)]
pub struct Config {
    pub port: u16,
    pub db: String,
    pub better_auth_secret: String,
}

impl Config {
    pub fn get() -> &'static LazyLock<Self> {
        &INSTANCE
    }
}

static INSTANCE: LazyLock<Config> = LazyLock::new(|| {
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

    let better_auth_secret = std::env::var("BETTER_AUTH_SECRET")
        .context("BETTER_AUTH_SECRET env var not found")
        .unwrap();

    Config {
        port,
        db,
        better_auth_secret,
    }
});
