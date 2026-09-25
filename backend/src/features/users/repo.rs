use anyhow::{Error, Result};
use chrono::NaiveDateTime;
use entity::{prelude::*, refresh_tokens, users};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, DeleteResult, EntityTrait};
use uuid::Uuid;

pub async fn find_user_by_username(
    db: &DatabaseConnection,
    username: &str,
) -> Result<Option<users::Model>> {
    Users::find_by_username(username)
        .one(db)
        .await
        .map_err(Error::new)
}

pub async fn create_user(
    db: &DatabaseConnection,
    username: String,
    hashed_password: String,
) -> Result<users::Model> {
    users::ActiveModel {
        username: ActiveValue::Set(username),
        password: ActiveValue::Set(hashed_password),
        ..Default::default()
    }
    .insert(db)
    .await
    .map_err(Error::new)
}

pub async fn create_refresh_token(
    db: &DatabaseConnection,
    user_id: Uuid,
    expires_at: NaiveDateTime,
) -> Result<refresh_tokens::Model> {
    refresh_tokens::ActiveModel {
        user_id: ActiveValue::Set(user_id),
        expires_at: ActiveValue::Set(expires_at),
        ..Default::default()
    }
    .insert(db)
    .await
    .map_err(Error::new)
}

pub async fn delete_refresh_token(db: &DatabaseConnection, token: Uuid) -> Result<DeleteResult> {
    RefreshTokens::delete_by_id(token)
        .exec(db)
        .await
        .map_err(Error::new)
}
