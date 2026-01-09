use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ChatHistories::Table)
                    .add_column(
                        ColumnDef::new(ChatHistories::CurrentPage)
                            .integer()
                            .not_null()
                            .default(1),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ChatHistories::Table)
                    .add_column(
                        ColumnDef::new(ChatHistories::ReadingSettings)
                            .string()
                            .null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ChatHistories::Table)
                    .drop_column(ChatHistories::CurrentPage)
                    .drop_column(ChatHistories::ReadingSettings)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum ChatHistories {
    Table,
    CurrentPage,
    ReadingSettings,
}
