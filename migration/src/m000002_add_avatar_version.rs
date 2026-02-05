//! 迁移：添加 avatar_version 列到 character_cards 表
//!
//! 用于封面浏览器缓存控制

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 添加 avatar_version 列，默认值为 1
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE character_cards ADD COLUMN avatar_version INTEGER NOT NULL DEFAULT 1;",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // SQLite 3.35.0+ 支持 DROP COLUMN
        manager
            .get_connection()
            .execute_unprepared("ALTER TABLE character_cards DROP COLUMN avatar_version;")
            .await?;

        Ok(())
    }
}
