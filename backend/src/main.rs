mod app;
mod error;
mod features;
mod settings;

use anyhow::{Context, Result};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, Database};
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{app::AppState, features::{auth::Auth, rooms::manager::RoomManager}, settings::Settings};

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

    let auth = Auth::get(db.clone()).await?;

    let room_manager = RoomManager::new();

    let app = app::router(AppState { db, auth, room_manager });

    let listener = TcpListener::bind(format!("0.0.0.0:{port}", port = settings.port))
        .await
        .unwrap();

    info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
