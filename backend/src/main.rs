mod app;
mod env;
mod error;
mod features;

use anyhow::{Context, Result};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, Database};
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::env::Env;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug,sqlx=error".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let env = Env::get();

    let db = Database::connect(ConnectOptions::new(&env.db).sqlx_logging(false).to_owned()).await?;
    Migrator::up(&db, None).await.context("")?;

    let app = app::router(db)?;

    let listener = TcpListener::bind(format!("127.0.0.1:{port}", port = env.port))
        .await
        .unwrap();

    info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
