use std::{env, sync::LazyLock};

use anyhow::Context;

#[derive(Debug)]
pub struct Config {
    pub port: u16,
    pub db: String,

    pub better_auth_secret: String,
    pub better_auth_base_url: String,
    pub better_auth_base_path: String,

    pub google_client_id: String,
    pub google_client_secret: String,
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

    let port: u16 = env::var("PORT")
        .unwrap_or_default()
        .parse::<u16>()
        .unwrap_or(3435);

    let db = env::var("DATABASE_URL")
        .context("DATABASE_URL env var not found")
        .unwrap();

    let better_auth_secret = env::var("BETTER_AUTH_SECRET")
        .context("BETTER_AUTH_SECRET env var not found")
        .unwrap();

    let better_auth_base_url =
        env::var("BETTER_AUTH_BASE_URL").unwrap_or_else(|_| "http://localhost:3435".to_string());

    let better_auth_base_path = "/auth".to_string();

    let google_client_id = env::var("GOOGLE_CLIENT_ID")
        .context("GOOGLE_CLIENT_ID env var not found")
        .unwrap();

    let google_client_secret = env::var("GOOGLE_CLIENT_SECRET")
        .context("GOOGLE_CLIENT_SECRET env var not found")
        .unwrap();

    Config {
        port,
        db,

        better_auth_secret,
        better_auth_base_url,
        better_auth_base_path,

        google_client_id,
        google_client_secret,
    }
});
