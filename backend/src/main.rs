mod app;
mod auth;
mod config;
mod error;
mod features;

use std::sync::Arc;

use anyhow::{Context, Result};
use better_auth::{AuthConfig, BetterAuth, seaorm::SeaOrmStore};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, Database};
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{app::AppState, auth::AppAuthSchema, config::Config};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug,sqlx=error".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::get();

    let db = Database::connect(
        ConnectOptions::new(&config.db)
            .sqlx_logging(false)
            .to_owned(),
    )
    .await?;
    Migrator::up(&db, None).await.context("")?;

    let auth_config = AuthConfig::new(&config.better_auth_secret).base_path("/auth");
    let store = SeaOrmStore::<AppAuthSchema>::new(auth_config.clone(), db.clone());

    let auth = Arc::new(
        BetterAuth::<AppAuthSchema>::new(auth_config)
            .store(store)
            .build()
            .await?,
    );

    let app = app::router(AppState { db, auth });

    let listener = TcpListener::bind(format!("127.0.0.1:{port}", port = config.port))
        .await
        .unwrap();

    info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
