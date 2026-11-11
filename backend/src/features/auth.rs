use std::sync::Arc;

use anyhow::Result;
use better_auth::{
    AuthConfig, AuthSchema, BetterAuth,
    config::SameSite,
    plugins::{OAuthPlugin, SessionManagementPlugin, oauth::OAuthProvider},
    seaorm::SeaOrmStore,
};
use entity::{accounts, sessions, users, verifications};
use sea_orm::DatabaseConnection;

use crate::settings::Settings;

pub struct AppAuthSchema;

impl AuthSchema for AppAuthSchema {
    type User = users::Model;
    type Session = sessions::Model;
    type Account = accounts::Model;
    type Verification = verifications::Model;
}

pub struct Auth;

impl Auth {
    pub async fn get(db: DatabaseConnection) -> Result<Arc<BetterAuth<AppAuthSchema>>> {
        let settings = Settings::get();

        let mut auth_config = AuthConfig::new(&settings.better_auth_secret)
            .base_url(&settings.better_auth_base_url)
            .base_path(&settings.better_auth_base_path)
            .trusted_origins(vec!["http://localhost:3000".to_string()]);
        auth_config.session.cookie_same_site = SameSite::None;

        let store = SeaOrmStore::<AppAuthSchema>::new(auth_config.clone(), db);

        Ok(Arc::new(
            BetterAuth::<AppAuthSchema>::new(auth_config)
                .store(store)
                .plugin(SessionManagementPlugin::new())
                .plugin(OAuthPlugin::new().add_provider(
                    "google",
                    OAuthProvider::google(
                        &settings.google_client_id,
                        &settings.google_client_secret,
                    ),
                ))
                .build()
                .await?,
        ))
    }
}
