use sea_orm::entity::prelude::DateTimeUtc as DateTimeWithTimeZone;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "rooms")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub owner_id: String,
    #[sea_orm(unique)]
    pub join_code: String,
    pub state_version: i64,
    pub current_item_id: Option<String>,
    pub playback_status: String,
    pub playback_position_ms: i64,
    pub playback_changed_at: DateTimeWithTimeZone,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
