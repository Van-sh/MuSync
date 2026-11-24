use sea_orm::entity::prelude::DateTimeUtc as DateTimeWithTimeZone;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "room_commands")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub room_id: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub command_id: String,
    pub user_id: String,
    pub command_type: String,
    pub command_hash: String,
    pub result_version: i64,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
