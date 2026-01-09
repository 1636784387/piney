use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ChatHistories::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ChatHistories::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ChatHistories::CardId).uuid().not_null())
                    .col(ColumnDef::new(ChatHistories::FileName).string().not_null())
                    .col(
                        ColumnDef::new(ChatHistories::DisplayName)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ChatHistories::FileSize)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(ChatHistories::Format).string().not_null())
                    .col(
                        ColumnDef::new(ChatHistories::Progress)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(ChatHistories::CreatedAt)
                            .date_time()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ChatHistories::UpdatedAt)
                            .date_time()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ChatHistories::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ChatHistories {
    Table,
    Id,
    CardId,
    FileName,
    DisplayName,
    FileSize,
    Format,
    Progress,
    CreatedAt,
    UpdatedAt,
}
