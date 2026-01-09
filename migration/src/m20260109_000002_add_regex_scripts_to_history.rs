use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // SQLite doesn't support adding a column with a default value that is not constant in the same way,
        // but for JSON text '[]' is fine.
        manager
            .alter_table(
                Table::alter()
                    .table(ChatHistories::Table)
                    .add_column(
                        ColumnDef::new(ChatHistories::RegexScripts)
                            .text()
                            .not_null()
                            .default("[]"),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // SQLite doesn't support dropping columns easily in alter table before newer versions,
        // but SeaORM usually handles what it can. If strict mode, might need recreation.
        // For now, standard drop column.
        manager
            .alter_table(
                Table::alter()
                    .table(ChatHistories::Table)
                    .drop_column(ChatHistories::RegexScripts)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum ChatHistories {
    Table,
    RegexScripts,
}
