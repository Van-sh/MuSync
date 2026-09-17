use anyhow::{Error, Result};
use entity::{prelude::*, users};
use sea_orm::{ActiveValue, DatabaseConnection, EntityTrait};

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
    let user = users::ActiveModel {
        username: ActiveValue::Set(username),
        password: ActiveValue::Set(hashed_password),
        ..Default::default()
    };

    Users::insert(user)
        .exec_with_returning(db)
        .await
        .map_err(Error::new)
}
