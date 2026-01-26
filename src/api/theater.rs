//! 小剧场 API 模块
//!
//! 提供小剧场的增删改查、导入导出功能

use crate::entities::{prelude::*, theater};
use axum::{
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::{Json, Response},
};
use chrono::Utc;
use sea_orm::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ==================== DTO ====================

/// 小剧场列表项 DTO
#[derive(Serialize)]
pub struct TheaterDto {
    pub id: Uuid,
    pub title: String,
    pub category: String,
    pub desc: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<theater::Model> for TheaterDto {
    fn from(m: theater::Model) -> Self {
        Self {
            id: m.id,
            title: m.title,
            category: m.category,
            desc: m.description,
            content: m.content,
            created_at: m.created_at.and_utc().to_rfc3339(),
            updated_at: m.updated_at.and_utc().to_rfc3339(),
        }
    }
}

/// 分页响应
#[derive(Serialize)]
pub struct TheaterListResponse {
    pub items: Vec<TheaterDto>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
    pub total_pages: u64,
}

/// 列表查询参数
#[derive(Deserialize)]
pub struct ListQuery {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub search: Option<String>,
    pub category: Option<String>,
}

/// 新建/更新请求
#[derive(Deserialize)]
pub struct TheaterPayload {
    pub title: Option<String>,
    pub category: Option<String>,
    pub desc: Option<String>,
    pub content: Option<String>,
}

/// 导入结果
#[derive(Serialize)]
pub struct ImportResult {
    pub total: usize,
    pub success: usize,
    pub failed: usize,
}

/// 导出查询参数
#[derive(Deserialize)]
pub struct ExportQuery {
    pub ids: Option<String>, // 逗号分隔的 ID 列表
}

// ==================== API Handlers ====================

/// 获取小剧场列表（分页、搜索、分类筛选）
pub async fn list_theaters(
    State(db): State<DatabaseConnection>,
    Query(query): Query<ListQuery>,
) -> Result<Json<TheaterListResponse>, (StatusCode, String)> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(50).min(100);

    let mut condition = Condition::all();

    // 分类筛选
    if let Some(cat) = &query.category {
        if !cat.is_empty() {
            condition = condition.add(theater::Column::Category.eq(cat.clone()));
        }
    }

    // 搜索（标题、简介、内容）
    if let Some(search) = &query.search {
        if !search.is_empty() {
            let pattern = format!("%{}%", search);
            condition = condition.add(
                Condition::any()
                    .add(theater::Column::Title.like(&pattern))
                    .add(theater::Column::Description.like(&pattern))
                    .add(theater::Column::Content.like(&pattern)),
            );
        }
    }

    // 总数
    let total = Theater::find()
        .filter(condition.clone())
        .count(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let total_pages = (total as f64 / page_size as f64).ceil() as u64;

    // 分页查询（排序：updated_at DESC, created_at DESC）
    let items = Theater::find()
        .filter(condition)
        .order_by_desc(theater::Column::UpdatedAt)
        .order_by_desc(theater::Column::CreatedAt)
        .offset(((page - 1) * page_size) as u64)
        .limit(page_size as u64)
        .all(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let items: Vec<TheaterDto> = items.into_iter().map(TheaterDto::from).collect();

    Ok(Json(TheaterListResponse {
        items,
        total,
        page,
        page_size,
        total_pages,
    }))
}

/// 获取所有分类
pub async fn list_categories(
    State(db): State<DatabaseConnection>,
) -> Result<Json<Vec<String>>, (StatusCode, String)> {
    // 使用原生SQL获取去重分类
    let categories: Vec<String> = Theater::find()
        .select_only()
        .column(theater::Column::Category)
        .distinct()
        .into_tuple::<String>()
        .all(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(categories))
}

/// 新建小剧场
pub async fn create_theater(
    State(db): State<DatabaseConnection>,
    Json(payload): Json<TheaterPayload>,
) -> Result<Json<TheaterDto>, (StatusCode, String)> {
    let title = payload
        .title
        .ok_or((StatusCode::BAD_REQUEST, "标题不能为空".to_string()))?;
    let desc = payload
        .desc
        .ok_or((StatusCode::BAD_REQUEST, "简介不能为空".to_string()))?;
    let content = payload
        .content
        .ok_or((StatusCode::BAD_REQUEST, "内容不能为空".to_string()))?;
    let category = payload.category.unwrap_or_else(|| "未分类".to_string());

    let now = Utc::now().naive_utc();
    let theater = theater::ActiveModel {
        id: Set(Uuid::new_v4()),
        title: Set(title),
        category: Set(category),
        description: Set(desc),
        content: Set(content),
        created_at: Set(now),
        updated_at: Set(now),
    };

    let saved = theater
        .insert(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(TheaterDto::from(saved)))
}

/// 获取单个小剧场
pub async fn get_theater(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<Json<TheaterDto>, (StatusCode, String)> {
    let theater = Theater::find_by_id(id)
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "小剧场不存在".to_string()))?;

    Ok(Json(TheaterDto::from(theater)))
}

/// 更新小剧场
pub async fn update_theater(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
    Json(payload): Json<TheaterPayload>,
) -> Result<Json<TheaterDto>, (StatusCode, String)> {
    let theater = Theater::find_by_id(id)
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "小剧场不存在".to_string()))?;

    let mut active: theater::ActiveModel = theater.into();

    if let Some(title) = payload.title {
        active.title = Set(title);
    }
    if let Some(category) = payload.category {
        active.category = Set(category);
    }
    if let Some(desc) = payload.desc {
        active.description = Set(desc);
    }
    if let Some(content) = payload.content {
        active.content = Set(content);
    }
    active.updated_at = Set(Utc::now().naive_utc());

    let updated = active
        .update(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(TheaterDto::from(updated)))
}

/// 删除小剧场
pub async fn delete_theater(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let result = Theater::delete_by_id(id)
        .exec(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if result.rows_affected == 0 {
        return Err((StatusCode::NOT_FOUND, "小剧场不存在".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

/// 批量删除小剧场
#[derive(Deserialize)]
pub struct BatchDeletePayload {
    pub ids: Vec<Uuid>,
}

pub async fn batch_delete_theaters(
    State(db): State<DatabaseConnection>,
    Json(payload): Json<BatchDeletePayload>,
) -> Result<StatusCode, (StatusCode, String)> {
    if payload.ids.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "没有提供要删除的 ID".to_string()));
    }

    Theater::delete_many()
        .filter(theater::Column::Id.is_in(payload.ids))
        .exec(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

/// 导入小剧场（TXT 格式）
pub async fn import_theaters(
    State(db): State<DatabaseConnection>,
    mut multipart: Multipart,
) -> Result<Json<ImportResult>, (StatusCode, String)> {
    let mut file_content = String::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            let data = field
                .bytes()
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            file_content = String::from_utf8_lossy(&data).to_string();
        }
    }

    if file_content.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "文件内容为空".to_string()));
    }

    // 解析小剧场
    let theaters = parse_theater_txt(&file_content);
    let total = theaters.len();
    let mut success = 0;
    let mut failed = 0;

    let now = Utc::now().naive_utc();

    for t in theaters {
        let theater = theater::ActiveModel {
            id: Set(Uuid::new_v4()),
            title: Set(t.title),
            category: Set(t.category),
            description: Set(t.desc),
            content: Set(t.content),
            created_at: Set(now),
            updated_at: Set(now),
        };

        match theater.insert(&db).await {
            Ok(_) => success += 1,
            Err(_) => failed += 1,
        }
    }

    Ok(Json(ImportResult {
        total,
        success,
        failed,
    }))
}

/// 解析后的小剧场结构
struct ParsedTheater {
    title: String,
    category: String,
    desc: String,
    content: String,
}

/// 解析 TXT 格式的小剧场
fn parse_theater_txt(content: &str) -> Vec<ParsedTheater> {
    let mut result = Vec::new();

    // 规范化换行符
    let normalized = content.replace("\r\n", "\n").replace("\r", "\n");
    let lines: Vec<&str> = normalized.lines().collect();

    // 找到所有小剧场的起始位置
    // 规则：当前行以 "###" 开头，且下一行以 "Title:" 开头
    let mut theater_starts: Vec<usize> = Vec::new();

    for i in 0..lines.len() {
        let line = lines[i].trim();
        let lower = line.to_lowercase();

        // 当前行以 ### 开头
        if lower.starts_with("###") {
            // 检查下一行是否以 Title: 开头
            if i + 1 < lines.len() {
                let next_line = lines[i + 1].trim().to_lowercase();
                if next_line.starts_with("title:") {
                    theater_starts.push(i);
                }
            }
        }
    }

    // 如果没找到任何小剧场标记，尝试直接解析（可能文件只有一个小剧场且没有###头）
    if theater_starts.is_empty() {
        if let Some(theater) = parse_single_theater(&lines, 0, lines.len()) {
            result.push(theater);
        }
        return result;
    }

    // 按找到的位置分割并解析每个小剧场
    for (idx, &start) in theater_starts.iter().enumerate() {
        let end = if idx + 1 < theater_starts.len() {
            theater_starts[idx + 1]
        } else {
            lines.len()
        };

        if let Some(theater) = parse_single_theater(&lines, start, end) {
            result.push(theater);
        }
    }

    result
}

/// 解析单个小剧场（从 lines[start..end] 范围内解析）
fn parse_single_theater(lines: &[&str], start: usize, end: usize) -> Option<ParsedTheater> {
    let mut title = String::new();
    let mut category = "未分类".to_string();
    let mut desc = String::new();
    let mut content_lines: Vec<&str> = Vec::new();
    let mut in_content = false;
    let mut found_title = false;

    for i in start..end {
        let line = lines[i];
        let trimmed = line.trim();
        let lower = trimmed.to_lowercase();

        // 跳过 "###" 开头的行（小剧场标记行）
        if lower.starts_with("###") {
            continue;
        }

        if !in_content {
            if lower.starts_with("title:") {
                title = trimmed
                    .get(6..) // "Title:" 长度为 6
                    .unwrap_or("")
                    .trim()
                    .to_string();
                found_title = true;
            } else if lower.starts_with("category:") {
                category = trimmed
                    .get(9..) // "Category:" 长度为 9
                    .unwrap_or("")
                    .trim()
                    .to_string();
                if category.is_empty() {
                    category = "未分类".to_string();
                }
            } else if lower.starts_with("desc:") {
                desc = trimmed
                    .get(5..) // "Desc:" 长度为 5
                    .unwrap_or("")
                    .trim()
                    .to_string();
            } else if found_title && trimmed.is_empty() {
                // 找到 Title 后的空行，开始读取内容
                in_content = true;
            } else if found_title
                && !trimmed.is_empty()
                && !lower.starts_with("category:")
                && !lower.starts_with("desc:")
            {
                // 如果已有 title，遇到非元数据的非空行，开始读取内容
                in_content = true;
                content_lines.push(line);
            }
        } else {
            // 在内容区域中
            content_lines.push(line);
        }
    }

    // 提取 Content（保留原始格式，只去除首尾空白）
    let content = content_lines.join("\n").trim().to_string();

    // 验证必填字段：必须有 title
    if title.is_empty() {
        return None;
    }

    // desc 为空时使用 content 的前 20 个字符
    let final_desc = if desc.is_empty() && !content.is_empty() {
        content.chars().take(20).collect::<String>()
    } else {
        desc
    };

    // 必须有内容
    if content.is_empty() {
        return None;
    }

    // 如果 desc 仍为空，使用默认值
    let final_desc = if final_desc.is_empty() {
        "无简介".to_string()
    } else {
        final_desc
    };

    Some(ParsedTheater {
        title,
        category,
        desc: final_desc,
        content,
    })
}

/// 导出小剧场（TXT 格式）
pub async fn export_theaters(
    State(db): State<DatabaseConnection>,
    Query(query): Query<ExportQuery>,
) -> Result<Response, (StatusCode, String)> {
    let theaters = if let Some(ids_str) = &query.ids {
        // 导出指定 ID
        let ids: Vec<Uuid> = ids_str
            .split(',')
            .filter_map(|s| Uuid::parse_str(s.trim()).ok())
            .collect();

        if ids.is_empty() {
            return Err((StatusCode::BAD_REQUEST, "无效的 ID 列表".to_string()));
        }

        Theater::find()
            .filter(theater::Column::Id.is_in(ids))
            .order_by_desc(theater::Column::UpdatedAt)
            .all(&db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    } else {
        // 导出全部（流式处理大数据量）
        Theater::find()
            .order_by_desc(theater::Column::UpdatedAt)
            .all(&db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    };

    // 生成 TXT 内容
    let mut output = String::new();
    for (i, t) in theaters.iter().enumerate() {
        if i > 0 {
            output.push_str("\n");
        }
        output.push_str("### Title\n");
        output.push_str(&format!("Title: {}\n", t.title));
        output.push_str(&format!("Category: {}\n", t.category));
        output.push_str(&format!("Desc: {}\n", t.description));
        output.push_str("\n");
        output.push_str(&t.content);
        output.push_str("\n");
    }

    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/plain; charset=utf-8")
        .header(
            "Content-Disposition",
            format!(
                "attachment; filename=\"theaters_export_{}.txt\"",
                Utc::now().format("%Y%m%d_%H%M%S")
            ),
        )
        .body(Body::from(output))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(response)
}
