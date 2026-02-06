//! 迁移：添加 avatar_version 列到 character_cards 表
//!
//! 用于封面浏览器缓存控制

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 检查列是否已存在（SQLite 不支持 IF NOT EXISTS）
        let conn = manager.get_connection();
        let result = conn
            .query_all(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                "SELECT COUNT(*) as cnt FROM pragma_table_info('character_cards') WHERE name='avatar_version'".to_string(),
            ))
            .await?;

        // 如果列不存在，添加它
        if let Some(row) = result.first() {
            let count: i32 = row.try_get("", "cnt").unwrap_or(0);
            if count == 0 {
                conn.execute_unprepared(
                    "ALTER TABLE character_cards ADD COLUMN avatar_version INTEGER NOT NULL DEFAULT 1;",
                )
                .await?;
            }
            // 如果列已存在，跳过（幂等迁移）
        }

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
