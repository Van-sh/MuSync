use anyhow::{Error, Result};
use entity::{prelude::*, users};
use sea_orm::DatabaseConnection;

pub async fn find_user_by_username(
    db: &DatabaseConnection,
    username: &str,
) -> Result<Option<users::Model>> {
    Users::find_by_username(username)
        .one(db)
        .await
        .map_err(Error::new)
}
