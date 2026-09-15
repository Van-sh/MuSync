use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let create_function = r#"
            CREATE OR REPLACE FUNCTION update_updated_at_column()
            RETURNS TRIGGER AS $$
            BEGIN
                NEW.updated_at = CURRENT_TIMESTAMP;
                RETURN NEW;
            END;
            $$ language 'plpgsql';
        "#;

        manager
            .get_connection()
            .execute_unprepared(create_function)
            .await?;

        manager
            .create_table(
                Table::create()
                    .table("users")
                    .if_not_exists()
                    .col(uuid("id").primary_key())
                    .col(string_uniq("username"))
                    .col(string("password"))
                    .col(date_time_default_now("created_at"))
                    .col(date_time_default_now("updated_at"))
                    .to_owned(),
            )
            .await?;

        // Create the trigger on your table
        let create_trigger = r#"
            CREATE TRIGGER update_users_updated_at_trigger
                BEFORE UPDATE ON "users"
                FOR EACH ROW
                EXECUTE FUNCTION update_updated_at_column();
        "#;

        manager
            .get_connection()
            .execute_unprepared(create_trigger)
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let drop_trigger = r#"
            DROP TRIGGER IF EXISTS update_users_updated_at_trigger ON your_table_name;
        "#;

        manager
            .get_connection()
            .execute_unprepared(drop_trigger)
            .await?;

        manager
            .drop_table(Table::drop().table("users").to_owned())
            .await?;

        // Drop the function
        let drop_function = r#"
            DROP FUNCTION IF EXISTS update_updated_at_column();
        "#;

        manager
            .get_connection()
            .execute_unprepared(drop_function)
            .await?;

        Ok(())
    }
}
