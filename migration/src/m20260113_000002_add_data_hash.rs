use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(CharacterCards::Table)
                    .add_column(ColumnDef::new(CharacterCards::DataHash).text().null())
                    .to_owned(),
            )
            .await?;

        // 创建索引以加速查重
        manager
            .create_index(
                Index::create()
                    .name("idx_character_cards_data_hash")
                    .table(CharacterCards::Table)
                    .col(CharacterCards::DataHash)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_character_cards_data_hash")
                    .table(CharacterCards::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(CharacterCards::Table)
                    .drop_column(CharacterCards::DataHash)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum CharacterCards {
    Table,
    DataHash,
}
