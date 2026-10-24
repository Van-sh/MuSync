mod app;
mod auth;
mod config;
mod error;
mod features;

use anyhow::{Context, Result};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, Database};
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{app::AppState, config::Settings};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug,sqlx=error".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let settings = Settings::get();

    let db = Database::connect(
        ConnectOptions::new(&settings.database_url)
            .sqlx_logging(false)
            .to_owned(),
    )
    .await?;
    Migrator::up(&db, None).await.context("")?;

    let auth = auth::get_auth(db.clone()).await?;

    let app = app::router(AppState { db, auth });

    let listener = TcpListener::bind(format!("0.0.0.0:{port}", port = settings.port))
        .await
        .unwrap();

    info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
