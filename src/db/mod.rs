//! 数据库模块
//!
//! 管理数据库连接和迁移

pub mod connection;

use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement};
use sea_orm_migration::MigratorTrait;
use tracing::info;

/// 检测并清理旧版迁移记录
///
/// 如果检测到 seaql_migrations 表中存在旧版迁移记录（非 m000001 开头），
/// 自动清空这些记录，让新的 v1 合并脚本可以正常运行。
async fn auto_upgrade_migrations(db: &DatabaseConnection) -> anyhow::Result<()> {
    // 检查 seaql_migrations 表是否存在
    let table_exists = db
        .execute(Statement::from_string(
            DbBackend::Sqlite,
            "SELECT name FROM sqlite_master WHERE type='table' AND name='seaql_migrations';"
                .to_owned(),
        ))
        .await;

    if table_exists.is_err() {
        return Ok(()); // 表不存在，是全新数据库，无需清理
    }

    // 检查是否有旧版迁移记录（非 m000001 开头的）
    let old_migrations = db
        .query_all(Statement::from_string(
            DbBackend::Sqlite,
            "SELECT version FROM seaql_migrations WHERE version NOT LIKE 'm000001%';".to_owned(),
        ))
        .await?;

    if !old_migrations.is_empty() {
        info!(
            "🔄 检测到 {} 条旧版迁移记录，正在自动升级到 v1.0...",
            old_migrations.len()
        );

        // 清空旧的迁移记录
        db.execute(Statement::from_string(
            DbBackend::Sqlite,
            "DELETE FROM seaql_migrations;".to_owned(),
        ))
        .await?;

        info!("✅ 旧版迁移记录已清理，将使用新的合并脚本");
    }

    Ok(())
}

/// 初始化数据库连接
pub async fn init_database() -> anyhow::Result<DatabaseConnection> {
    // 获取数据目录
    let data_path = crate::utils::paths::get_data_dir();

    // 确保数据目录存在
    if !data_path.exists() {
        std::fs::create_dir_all(&data_path)?;
        info!("创建数据目录: {:?}", data_path);
    }

    // 确保子目录存在
    // Optimization: Only create directories that are actually used
    for subdir in ["cards", "uploads"] {
        let subdir_path = data_path.join(subdir);
        if !subdir_path.exists() {
            std::fs::create_dir_all(&subdir_path)?;
        }
    }

    // 数据库文件路径
    let db_path = data_path.join("piney.db");
    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

    info!("连接数据库: {}", db_url);

    // 连接数据库
    let db = Database::connect(&db_url).await?;

    // 开启 WAL 模式以提高并发性能，并设置 busy_timeout 防止锁竞争导致 500
    db.execute(Statement::from_string(
        DbBackend::Sqlite,
        "PRAGMA journal_mode=WAL;".to_owned(),
    ))
    .await?;

    db.execute(Statement::from_string(
        DbBackend::Sqlite,
        "PRAGMA busy_timeout=5000;".to_owned(),
    ))
    .await?;

    db.execute(Statement::from_string(
        DbBackend::Sqlite,
        "PRAGMA foreign_keys = ON;".to_owned(),
    ))
    .await?;

    // 自动升级：检测并清理旧版迁移记录
    auto_upgrade_migrations(&db).await?;

    // 运行迁移
    info!("检查数据库迁移...");
    migration::Migrator::up(&db, None).await?;
    info!("数据库迁移完成");

    Ok(db)
}
