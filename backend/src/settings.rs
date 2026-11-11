use std::sync::LazyLock;

use anyhow::Context;
use config::{Config, Environment};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    #[serde(default = "default_port")]
    pub port: u16,
    pub database_url: String,

    pub better_auth_secret: String,
    #[serde(default = "default_better_auth_base_url")]
    pub better_auth_base_url: String,
    #[serde(skip, default = "default_better_auth_base_path")]
    pub better_auth_base_path: String,

    pub google_client_id: String,
    pub google_client_secret: String,
}

impl Settings {
    pub fn get() -> &'static LazyLock<Self> {
        static INSTANCE: LazyLock<Settings> = LazyLock::new(|| {
            dotenvy::dotenv()
                .map_err(|_| tracing::debug!(".env not found"))
                .ok();

            let s = Config::builder()
                .add_source(Environment::default())
                .build()
                .context("Something went wrong while initialising Config")
                .unwrap();

            s.try_deserialize()
                .context("Failed to parse Settings")
                .unwrap()
        });

        &INSTANCE
    }
}

fn default_port() -> u16 {
    3435
}

fn default_better_auth_base_url() -> String {
    "http://localhost:3435".to_string()
}

fn default_better_auth_base_path() -> String {
    "/auth".to_string()
}
