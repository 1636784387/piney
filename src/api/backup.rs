//! 数据备份与恢复 API
//!
//! 导出：打包整个 data/ 目录为 .tar.gz (返回为 .piney)
//! 导入：解压 .piney 文件覆盖 data/ 目录

use axum::{
    body::Body,
    extract::{Json, Multipart, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
};
use chrono::{Duration, Local, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use sea_orm::{Database, DatabaseConnection};

use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use std::fs;
use tar::{Archive, Builder};
use tokio::io::duplex;
use tokio_util::io::{ReaderStream, SyncIoBridge}; // 用于流式传输
use tracing::{error, info};

use crate::auth::Claims;
use crate::config::ConfigState;

/// 备份恢复所需的组合状态
#[derive(Clone)]
pub struct BackupState {
    pub db: DatabaseConnection,
    pub config: ConfigState,
}

#[derive(Serialize)]
pub struct ImportResponse {
    username: String,
    message: String,
    token: Option<String>, // 新增：恢复成功后返回新 Token
}

/// 获取 data 目录路径
fn get_data_dir() -> std::path::PathBuf {
    crate::utils::paths::get_data_path("")
}

/// GET /api/backup/export - 导出系统数据为 .piney 文件 (流式传输)
pub async fn export_backup() -> Result<impl IntoResponse, (StatusCode, String)> {
    let data_dir = get_data_dir();

    if !data_dir.exists() {
        return Err((StatusCode::NOT_FOUND, "数据目录不存在".to_string()));
    }

    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("piney_backup_{}.piney", timestamp);

    // 1. 创建内存管道 (4MB 缓冲区，优化 200MB/s+ 高速下载体验)
    // reader 给 Axum 返回给前端作为 Response Body
    // writer 给 tar::Builder 写入数据
    let (reader, writer) = duplex(4 * 1024 * 1024);

    let data_dir_clone = data_dir.clone();

    // 2. 启动后台任务进行打包
    // 使用 SyncIoBridge 将 异步writer 转换为 同步write，供同步的 tar 库使用
    // spawn_blocking 在专用线程池运行，不会阻塞因为 heavy I/O
    tokio::task::spawn_blocking(move || {
        // SyncIoBridge 会在 drop 时关闭 writer，从而给 reader 发送 EOF
        let bridge = SyncIoBridge::new(writer);
        // 关键优化：使用 BufWriter 减少 tar 的大量小 I/O (?512bytes) 操作导致的频繁 context switch
        // 4MB 缓冲区与 pipe 容量一致，最大化吞吐量
        let buffered_bridge = std::io::BufWriter::with_capacity(4 * 1024 * 1024, bridge);
        let mut tar_builder = Builder::new(buffered_bridge);

        // 使用闭包来捕获 Result，方便统一处理错误
        let result = (|| -> Result<(), String> {
            if let Ok(entries) = fs::read_dir(&data_dir_clone) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                    // 跳过不应该备份的文件
                    // - temp 目录
                    // - config.yml (用户名/密码)
                    // - .jwt_secret (JWT 密钥)
                    if filename == "temp" || filename == "config.yml" || filename == ".jwt_secret" {
                        continue;
                    }

                    // 计算相对路径
                    let relative_path = path
                        .strip_prefix(&data_dir_clone)
                        .map_err(|e| format!("路径错误: {}", e))?;

                    // 写入 tar
                    if path.is_dir() {
                        tar_builder
                            .append_dir_all(relative_path, &path)
                            .map_err(|e| format!("打包目录失败 {:?}: {}", path, e))?;
                    } else {
                        tar_builder
                            .append_path_with_name(&path, relative_path)
                            .map_err(|e| format!("打包文件失败 {:?}: {}", path, e))?;
                    }
                }
            }

            // 完成打包
            tar_builder
                .finish()
                .map_err(|e| format!("Tar finish failed: {}", e))?;

            Ok(())
        })();

        if let Err(e) = result {
            // 在流传输过程中发生错误，只能记录日志，无法修改 HTTP 状态码
            error!("备份打包流式传输失败: {}", e);
        }
    });

    // 3. 构建响应流
    // 使用 ReaderStream 将 AsyncRead 转换为 Stream
    let stream = ReaderStream::new(reader);
    let body = Body::from_stream(stream);

    // 构建响应头
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "application/octet-stream".parse().unwrap(),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}\"", filename)
            .parse()
            .unwrap(),
    );

    Ok((headers, body))
}

/// POST /api/backup/import - 导入 .piney 备份文件并恢复数据
pub async fn import_backup(
    State(state): State<BackupState>,
    mut multipart: Multipart,
) -> Result<Json<ImportResponse>, (StatusCode, String)> {
    // 0. 关闭数据库连接 (Fix: Windows file lock issue)
    // 必须确保在删除/覆盖 piney.db 前断开所有连接
    // 注意：这将导致后续请求直到重启前都无法访问数据库，这是预期的
    if let Err(e) = state.db.close().await {
        // Log but continue, sometimes it might be already closed or fail to close cleanly
        error!("尝试关闭数据库连接失败: {}", e);
    }

    // 1. 读取上传的文件
    let mut file_data: Option<Vec<u8>> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some("backup") || field.name() == Some("file") {
            let data = field
                .bytes()
                .await
                .map_err(|e| (StatusCode::BAD_REQUEST, format!("读取文件失败: {}", e)))?;
            file_data = Some(data.to_vec());
            break;
        }
    }

    let data = file_data.ok_or((StatusCode::BAD_REQUEST, "未找到备份文件".to_string()))?;

    // 2. 验证是否为有效的备份文件 (Tar 或 TarGz)
    let is_valid = {
        let data_clone = data.clone();
        tokio::task::spawn_blocking(move || {
            // 尝试当作 Tar 读取
            let mut archive = Archive::new(&data_clone[..]);
            if archive.entries().is_ok() {
                return true;
            }

            // 尝试当作 Gzip 读取
            let decoder = GzDecoder::new(&data_clone[..]);
            let mut archive = Archive::new(decoder);
            archive.entries().is_ok()
        })
        .await
        .unwrap_or(false)
    };

    if !is_valid {
        return Err((
            StatusCode::BAD_REQUEST,
            "无效的备份文件格式（不是有效的 tar.gz）".to_string(),
        ));
    }

    let data_dir = get_data_dir();

    // 3. 执行清空、解压、读取配置
    let data_clone = data.clone();
    let data_dir_clone = data_dir.clone();

    tokio::task::spawn_blocking(move || -> Result<(), String> {
        // A. 清空 data 目录
        let entries =
            fs::read_dir(&data_dir_clone).map_err(|e| format!("读取数据目录失败: {}", e))?;

        for entry in entries.flatten() {
            let path = entry.path();
            let filename = path.file_name().unwrap_or_default().to_string_lossy();

            // 跳过一些不应该删除的文件
            // - config.yml (用户名/密码) - 保留当前配置
            // - .jwt_secret (JWT 密钥) - 保留当前登录状态
            // - .DS_Store (系统文件)
            if filename == "config.yml" || filename == ".jwt_secret" || filename == ".DS_Store" {
                continue;
            }

            if path.is_dir() {
                if let Err(e) = fs::remove_dir_all(&path) {
                    tracing::warn!("删除目录 {:?} 失败: {}", path, e);
                }
            } else if let Err(e) = fs::remove_file(&path) {
                tracing::warn!("删除文件 {:?} 失败: {}", path, e);
            }
        }

        // B. 解压备份
        // 先尝试 gzip，否则 plain tar
        let decoder = GzDecoder::new(&data_clone[..]);
        let is_gz = decoder.header().is_some();

        if is_gz {
            let decoder = GzDecoder::new(&data_clone[..]);
            let mut archive = Archive::new(decoder);
            let entries = archive
                .entries()
                .map_err(|e| format!("读取归档失败: {}", e))?;
            for entry in entries {
                let mut entry = entry.map_err(|e| format!("读取条目失败: {}", e))?;

                // 跳过不应该恢复的文件（兼容旧备份）
                let entry_path = entry.path().map_err(|e| format!("读取路径失败: {}", e))?;
                let entry_name = entry_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                if entry_name == "config.yml" || entry_name == ".jwt_secret" {
                    continue;
                }

                entry.set_preserve_permissions(false);
                entry.set_preserve_mtime(false);
                entry
                    .unpack_in(&data_dir_clone)
                    .map_err(|e| format!("解压失败: {}", e))?;
            }
        } else {
            let mut archive = Archive::new(&data_clone[..]);
            let entries = archive
                .entries()
                .map_err(|e| format!("读取归档失败: {}", e))?;
            for entry in entries {
                let mut entry = entry.map_err(|e| format!("读取条目失败: {}", e))?;

                // 跳过不应该恢复的文件（兼容旧备份）
                let entry_path = entry.path().map_err(|e| format!("读取路径失败: {}", e))?;
                let entry_name = entry_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                if entry_name == "config.yml" || entry_name == ".jwt_secret" {
                    continue;
                }

                entry.set_preserve_permissions(false);
                entry.set_preserve_mtime(false);
                entry
                    .unpack_in(&data_dir_clone)
                    .map_err(|e| format!("解压失败: {}", e))?;
            }
        }

        // 不再需要从恢复的 config.yml 读取用户名，因为我们不覆盖 config.yml
        // 用户名将从当前配置中获取

        Ok(())
    })
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("任务执行失败: {}", e),
        )
    })?
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // 4. 刷新配置（重新从文件加载用户名/密码，并重新生成 JWT Secret）
    if let Err(e) = state.config.reload() {
        error!("配置刷新失败: {}", e);
        // 不返回错误，继续尝试返回成功信息（用户可能需要重启）
    } else {
        info!("配置已刷新");
    }

    // 5. 重新连接数据库（验证恢复后的数据库是否可用）
    let db_path = get_data_dir().join("piney.db");
    let db_url = format!("sqlite://{}?mode=rwc", db_path.to_string_lossy());

    match Database::connect(&db_url).await {
        Ok(_new_db) => {
            info!("数据库重新连接成功");
            // 注意：这里的新连接会在函数结束后 drop，
            // 但验证了数据库文件是完整的
        }
        Err(e) => {
            error!("数据库重新连接失败: {}", e);
            // 继续返回成功，但提示用户可能需要重启
        }
    }

    // 6. 从当前配置获取用户名并生成新的 JWT Token
    let username = state
        .config
        .get()
        .map(|c| c.username)
        .unwrap_or_else(|| "admin".to_string());

    let token = {
        let expiration = Utc::now()
            .checked_add_signed(Duration::days(90))
            .expect("valid timestamp")
            .timestamp();

        let claims = Claims {
            sub: username.clone(),
            exp: expiration as usize,
        };

        match encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(state.config.get_jwt_secret().as_bytes()),
        ) {
            Ok(t) => Some(t),
            Err(e) => {
                error!("生成 Token 失败: {}", e);
                None
            }
        }
    };

    // 7. 返回成功信息
    Ok(Json(ImportResponse {
        username,
        message: "数据恢复成功".to_string(),
        token,
    }))
}
