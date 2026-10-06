use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(string(Users::Id).primary_key())
                    .col(string_null(Users::Name))
                    .col(string_null(Users::Email).unique_key())
                    .col(boolean(Users::EmailVerified))
                    .col(string_null(Users::Image))
                    .col(date_time_default_now(Users::CreatedAt))
                    .col(date_time_default_now(Users::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Sessions::Table)
                    .if_not_exists()
                    .col(string(Sessions::Id).primary_key())
                    .col(date_time(Sessions::ExpiresAt))
                    .col(string_uniq(Sessions::Token))
                    .col(date_time_default_now(Sessions::CreatedAt))
                    .col(date_time_default_now(Sessions::UpdatedAt))
                    .col(string_null(Sessions::IpAddress))
                    .col(string_null(Sessions::UserAgent))
                    .col(string(Sessions::UserId))
                    .col(boolean(Sessions::Active))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(SessionsUserIdIdx.to_string())
                    .table(Sessions::Table)
                    .col(Sessions::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Accounts::Table)
                    .if_not_exists()
                    .col(string(Accounts::Id).primary_key())
                    .col(string(Accounts::AccountId))
                    .col(string(Accounts::ProviderId))
                    .col(string(Accounts::UserId))
                    .col(string_null(Accounts::AccessToken))
                    .col(string_null(Accounts::RefreshToken))
                    .col(string_null(Accounts::IdToken))
                    .col(date_time_null(Accounts::AccessTokenExpiresAt))
                    .col(date_time_null(Accounts::RefreshTokenExpiresAt))
                    .col(string_null(Accounts::Scope))
                    .col(string_null(Accounts::Password))
                    .col(date_time_default_now(Accounts::CreatedAt))
                    .col(date_time_default_now(Accounts::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(AccountsUserIdIdx.to_string())
                    .table(Accounts::Table)
                    .col(Accounts::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Verifications::Table)
                    .if_not_exists()
                    .col(string(Verifications::Id).primary_key())
                    .col(string(Verifications::Identifier))
                    .col(string(Verifications::Value))
                    .col(date_time(Verifications::ExpiresAt))
                    .col(date_time_default_now(Verifications::CreatedAt))
                    .col(date_time_default_now(Verifications::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(VerificationsIdentifierIdx.to_string())
                    .table(Verifications::Table)
                    .col(Verifications::Identifier)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name(VerificationsIdentifierIdx.to_string())
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Verifications::Table).to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name(AccountsUserIdIdx.to_string()).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Accounts::Table).to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name(SessionsUserIdIdx.to_string()).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Sessions::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    Name,
    Email,
    EmailVerified,
    Image,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Sessions {
    Table,
    Id,
    ExpiresAt,
    Token,
    CreatedAt,
    UpdatedAt,
    IpAddress,
    UserAgent,
    UserId,
    Active,
}
#[derive(DeriveIden)]
struct SessionsUserIdIdx;

#[derive(DeriveIden)]
enum Accounts {
    Table,
    Id,
    #[allow(clippy::enum_variant_names)]
    AccountId,
    ProviderId,
    UserId,
    AccessToken,
    RefreshToken,
    IdToken,
    AccessTokenExpiresAt,
    RefreshTokenExpiresAt,
    Scope,
    Password,
    CreatedAt,
    UpdatedAt,
}
#[derive(DeriveIden)]
struct AccountsUserIdIdx;

#[derive(DeriveIden)]
enum Verifications {
    Table,
    Id,
    Identifier,
    Value,
    ExpiresAt,
    CreatedAt,
    UpdatedAt,
}
#[derive(DeriveIden)]
struct VerificationsIdentifierIdx;
