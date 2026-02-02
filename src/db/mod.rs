//! 数据库模块
//!
//! 管理数据库连接和迁移

pub mod connection;

use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement};
use sea_orm_migration::MigratorTrait;
use tracing::info;

/// 检测并清理旧版或不完整的迁移记录
///
/// 处理以下情况：
/// 1. 存在旧版迁移记录（非 m000001 开头）- 清空让新脚本运行
/// 2. 存在 m000001 记录但缺少必要的表 - 清空让新脚本重新运行以补全缺失表
async fn auto_upgrade_migrations(db: &DatabaseConnection) -> anyhow::Result<()> {
    // 检查 seaql_migrations 表是否存在
    let migrations_table_exists = db
        .query_one(Statement::from_string(
            DbBackend::Sqlite,
            "SELECT name FROM sqlite_master WHERE type='table' AND name='seaql_migrations';"
                .to_owned(),
        ))
        .await?;

    if migrations_table_exists.is_none() {
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
        return Ok(());
    }

    // 检查是否存在 m000001 记录但缺少必要的表（不完整的迁移）
    let v1_migration = db
        .query_one(Statement::from_string(
            DbBackend::Sqlite,
            "SELECT version FROM seaql_migrations WHERE version LIKE 'm000001%';".to_owned(),
        ))
        .await?;

    if v1_migration.is_some() {
        // 检查 theaters 表是否存在（作为新表的代表）
        let theaters_exists = db
            .query_one(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT name FROM sqlite_master WHERE type='table' AND name='theaters';".to_owned(),
            ))
            .await?;

        if theaters_exists.is_none() {
            info!("🔧 检测到不完整的 v1 迁移（缺少 theaters 表），正在修复...");

            // 清空迁移记录，让新脚本重新运行以创建缺失的表
            db.execute(Statement::from_string(
                DbBackend::Sqlite,
                "DELETE FROM seaql_migrations;".to_owned(),
            ))
            .await?;

            info!("✅ 迁移记录已清理，新脚本将补全缺失的表");
        }
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

    // 关键修正：不再使用任何字符串拼接 URL，改用 Builder 模式直接配置
    // 这样可以彻底规避 Windows 下 PathBuf -> URL String 过程中的盘符/斜杠解析 BUG
    // 无论路径长什么样（盘符、中文、空格），sqlx 内部直接处理 PathBuf，不经过 URL parser

    use sea_orm::sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use sea_orm::SqlxSqliteConnector;

    // 配置连接选项 (避开 format!("sqlite:...") )
    let options = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(true);

    info!("连接数据库 (Builder模式): {:?}", db_path);

    // 创建连接池
    let pool = SqlitePoolOptions::new()
        .connect_with(options)
        .await
        .map_err(|e| anyhow::anyhow!("数据库连接失败: {}", e))?;

    // 转换为 SeaORM 连接
    let db = SqlxSqliteConnector::from_sqlx_sqlite_pool(pool);

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
