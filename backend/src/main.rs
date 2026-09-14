mod app;
mod env;
mod error;
mod features;

use tokio::net::TcpListener;
use tracing::debug;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::env::Env;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let env = Env::get();

    let listener = TcpListener::bind(format!("127.0.0.1:{port}", port = env.port))
        .await
        .unwrap();

    debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app::router()).await.unwrap();
}
