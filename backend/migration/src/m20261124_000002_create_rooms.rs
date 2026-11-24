use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Rooms::Table)
                    .if_not_exists()
                    .col(string(Rooms::Id).primary_key())
                    .col(string(Rooms::OwnerId))
                    .col(string_uniq(Rooms::JoinCode))
                    .col(big_integer(Rooms::StateVersion))
                    .col(string_null(Rooms::CurrentItemId))
                    .col(string(Rooms::PlaybackStatus))
                    .col(big_integer(Rooms::PlaybackPositionMs))
                    .col(timestamp_with_time_zone(Rooms::PlaybackChangedAt))
                    .col(timestamp_with_time_zone_default_now(Rooms::CreatedAt))
                    .col(timestamp_with_time_zone_default_now(Rooms::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(RoomMembers::Table)
                    .if_not_exists()
                    .col(string(RoomMembers::RoomId))
                    .col(string(RoomMembers::UserId))
                    .col(string(RoomMembers::Role))
                    .col(timestamp_with_time_zone_default_now(RoomMembers::JoinedAt))
                    .primary_key(
                        Index::create()
                            .col(RoomMembers::RoomId)
                            .col(RoomMembers::UserId),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(RoomQueueItems::Table)
                    .if_not_exists()
                    .col(string(RoomQueueItems::Id).primary_key())
                    .col(string(RoomQueueItems::RoomId))
                    .col(string(RoomQueueItems::VideoId))
                    .col(string(RoomQueueItems::AddedBy))
                    .col(integer(RoomQueueItems::SortOrder))
                    .col(timestamp_with_time_zone_default_now(
                        RoomQueueItems::CreatedAt,
                    ))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(RoomCommands::Table)
                    .if_not_exists()
                    .col(string(RoomCommands::RoomId))
                    .col(string(RoomCommands::CommandId))
                    .col(string(RoomCommands::UserId))
                    .col(string(RoomCommands::CommandType))
                    .col(string(RoomCommands::CommandHash))
                    .col(big_integer(RoomCommands::ResultVersion))
                    .col(timestamp_with_time_zone_default_now(
                        RoomCommands::CreatedAt,
                    ))
                    .primary_key(
                        Index::create()
                            .col(RoomCommands::RoomId)
                            .col(RoomCommands::CommandId),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(RoomCommands::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(RoomQueueItems::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(RoomMembers::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Rooms::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Rooms {
    Table,
    Id,
    OwnerId,
    JoinCode,
    StateVersion,
    CurrentItemId,
    PlaybackStatus,
    PlaybackPositionMs,
    PlaybackChangedAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum RoomMembers {
    Table,
    RoomId,
    UserId,
    Role,
    JoinedAt,
}

#[derive(DeriveIden)]
enum RoomQueueItems {
    Table,
    Id,
    RoomId,
    VideoId,
    AddedBy,
    SortOrder,
    CreatedAt,
}

#[derive(DeriveIden)]
enum RoomCommands {
    Table,
    RoomId,
    CommandId,
    UserId,
    CommandType,
    CommandHash,
    ResultVersion,
    CreatedAt,
}
