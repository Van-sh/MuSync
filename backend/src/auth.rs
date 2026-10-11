use std::sync::Arc;

use anyhow::Result;
use better_auth::{
    AuthConfig, AuthSchema, BetterAuth,
    plugins::{OAuthPlugin, oauth::OAuthProvider},
    seaorm::SeaOrmStore,
};
use entity::{accounts, sessions, users, verifications};
use sea_orm::DatabaseConnection;

use crate::config::Config;

pub struct AppAuthSchema;

impl AuthSchema for AppAuthSchema {
    type User = users::Model;
    type Session = sessions::Model;
    type Account = accounts::Model;
    type Verification = verifications::Model;
}

pub async fn get_auth(db: DatabaseConnection) -> Result<Arc<BetterAuth<AppAuthSchema>>> {
    let config = Config::get();

    let auth_config = AuthConfig::new(&config.better_auth_secret).base_path("/auth");
    let store = SeaOrmStore::<AppAuthSchema>::new(auth_config.clone(), db);

    Ok(Arc::new(
        BetterAuth::<AppAuthSchema>::new(auth_config)
            .store(store)
            .plugin(OAuthPlugin::new().add_provider(
                "google",
                OAuthProvider::google(&config.google_client_id, &config.google_client_secret),
            ))
            .build()
            .await?,
    ))
}
