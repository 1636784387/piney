//! 创建前端样式表
//!
//! 用于存储 AI 生成的前端样式（代码、正则、世界书）

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(FrontendStyle::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(FrontendStyle::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(FrontendStyle::Name).string().not_null())
                    .col(
                        ColumnDef::new(FrontendStyle::OriginalText)
                            .text()
                            .not_null()
                            .default(""),
                    )
                    .col(
                        ColumnDef::new(FrontendStyle::RegexPattern)
                            .text()
                            .not_null()
                            .default(""),
                    )
                    .col(
                        ColumnDef::new(FrontendStyle::HtmlCode)
                            .text()
                            .not_null()
                            .default(""),
                    )
                    .col(
                        ColumnDef::new(FrontendStyle::WorldinfoKey)
                            .text()
                            .not_null()
                            .default(""),
                    )
                    .col(
                        ColumnDef::new(FrontendStyle::WorldinfoContent)
                            .text()
                            .not_null()
                            .default(""),
                    )
                    .col(
                        ColumnDef::new(FrontendStyle::CreatedAt)
                            .date_time()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(FrontendStyle::UpdatedAt)
                            .date_time()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // 创建名称索引
        manager
            .create_index(
                Index::create()
                    .name("idx_frontend_style_name")
                    .table(FrontendStyle::Table)
                    .col(FrontendStyle::Name)
                    .to_owned(),
            )
            .await?;

        // 创建排序索引
        manager
            .create_index(
                Index::create()
                    .name("idx_frontend_style_sort")
                    .table(FrontendStyle::Table)
                    .col(FrontendStyle::UpdatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(FrontendStyle::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum FrontendStyle {
    Table,
    Id,
    Name,
    OriginalText,
    RegexPattern,
    HtmlCode,
    WorldinfoKey,
    WorldinfoContent,
    CreatedAt,
    UpdatedAt,
}
