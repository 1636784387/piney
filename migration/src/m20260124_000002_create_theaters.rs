//! 创建小剧场表
//!
//! 用于存储小剧场（AI 对话提示词）

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Theaters::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Theaters::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Theaters::Title).string().not_null())
                    .col(
                        ColumnDef::new(Theaters::Category)
                            .string()
                            .not_null()
                            .default("未分类"),
                    )
                    .col(ColumnDef::new(Theaters::Desc).text().not_null())
                    .col(ColumnDef::new(Theaters::Content).text().not_null())
                    .col(ColumnDef::new(Theaters::CreatedAt).date_time().not_null())
                    .col(ColumnDef::new(Theaters::UpdatedAt).date_time().not_null())
                    .to_owned(),
            )
            .await?;

        // 创建分类索引，便于按分类筛选
        manager
            .create_index(
                Index::create()
                    .name("idx_theaters_category")
                    .table(Theaters::Table)
                    .col(Theaters::Category)
                    .to_owned(),
            )
            .await?;

        // 创建排序索引（updated_at DESC, created_at DESC）
        manager
            .create_index(
                Index::create()
                    .name("idx_theaters_sort")
                    .table(Theaters::Table)
                    .col(Theaters::UpdatedAt)
                    .col(Theaters::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Theaters::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Theaters {
    Table,
    Id,
    Title,
    Category,
    Desc,
    Content,
    CreatedAt,
    UpdatedAt,
}
