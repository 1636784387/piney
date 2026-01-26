//! 创建快速回复表
//!
//! 快速回复文件与角色卡关联，支持多文件

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(QuickReplies::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(QuickReplies::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(QuickReplies::CardId).uuid().not_null())
                    .col(ColumnDef::new(QuickReplies::FileName).string().not_null())
                    .col(
                        ColumnDef::new(QuickReplies::DisplayName)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(QuickReplies::FileSize)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(QuickReplies::CreatedAt)
                            .date_time()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(QuickReplies::UpdatedAt)
                            .date_time()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_quick_replies_card_id")
                            .from(QuickReplies::Table, QuickReplies::CardId)
                            .to(CharacterCards::Table, CharacterCards::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // 创建索引以加速按角色卡查询
        manager
            .create_index(
                Index::create()
                    .name("idx_quick_replies_card_id")
                    .table(QuickReplies::Table)
                    .col(QuickReplies::CardId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(QuickReplies::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum QuickReplies {
    Table,
    Id,
    CardId,
    FileName,
    DisplayName,
    FileSize,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum CharacterCards {
    Table,
    Id,
}
