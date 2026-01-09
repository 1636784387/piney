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
                        ColumnDef::new(ChatHistories::SourceFileName)
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
                    .drop_column(ChatHistories::SourceFileName)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum ChatHistories {
    Table,
    SourceFileName,
}
