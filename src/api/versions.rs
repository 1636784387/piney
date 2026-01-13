use crate::entities::{character_card, character_versions, prelude::*};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateVersionRequest {
    pub version_number: String,
    pub note: Option<String>,
}

#[derive(Serialize)]
pub struct VersionResponse {
    pub id: Uuid,
    pub version_number: String,
    pub note: Option<String>,
    pub created_at: String, // ISO string
}

/// 创建版本 (快照当前角色卡)
pub async fn create_version(
    State(db): State<DatabaseConnection>,
    Path(card_id): Path<Uuid>,
    Json(payload): Json<CreateVersionRequest>,
) -> Result<Json<VersionResponse>, (StatusCode, String)> {
    // 1. 获取当前角色卡数据
    let card = CharacterCard::find_by_id(card_id)
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((
            StatusCode::NOT_FOUND,
            "Character card not found".to_string(),
        ))?;

    // 2. 创建版本记录
    let version = character_versions::ActiveModel {
        id: Set(Uuid::new_v4()),
        character_id: Set(card.id),
        version_number: Set(payload.version_number),
        note: Set(payload.note),
        data: Set(card.data), // Snapshot current data
        created_at: Set(chrono::Utc::now().naive_utc()),
    };

    let saved = version
        .insert(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(VersionResponse {
        id: saved.id,
        version_number: saved.version_number,
        note: saved.note,
        created_at: saved.created_at.to_string(),
    }))
}

///获取版本列表
pub async fn list_versions(
    State(db): State<DatabaseConnection>,
    Path(card_id): Path<Uuid>,
) -> Result<Json<Vec<VersionResponse>>, (StatusCode, String)> {
    let versions = CharacterVersion::find()
        .filter(character_versions::Column::CharacterId.eq(card_id))
        .order_by_desc(character_versions::Column::CreatedAt)
        .all(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let response = versions
        .into_iter()
        .map(|v| VersionResponse {
            id: v.id,
            version_number: v.version_number,
            note: v.note,
            created_at: v.created_at.to_string(),
        })
        .collect();

    Ok(Json(response))
}

/// 恢复版本
pub async fn restore_version(
    State(db): State<DatabaseConnection>,
    Path((card_id, version_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, String)> {
    // 1. 获取目标版本数据
    let version = CharacterVersion::find_by_id(version_id)
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Version not found".to_string()))?;

    // 安全检查：确保版本属于该角色
    if version.character_id != card_id {
        return Err((StatusCode::BAD_REQUEST, "Version mismatch".to_string()));
    }

    // 2. 获取当前角色卡
    let card = CharacterCard::find_by_id(card_id)
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((
            StatusCode::NOT_FOUND,
            "Character card not found".to_string(),
        ))?;

    // 3. 覆盖数据
    let mut card_active: character_card::ActiveModel = card.into();

    // Parse the JSON snapshot to extract denormalized fields
    let json: serde_json::Value = serde_json::from_str(&version.data).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to parse version snapshot: {}", e),
        )
    })?;

    // Normalize V2/V3 structure (logic from src/api/cards.rs)
    let card_data = if let Some(d) = json.get("data") {
        if d.is_object() {
            d
        } else {
            &json
        }
    } else {
        &json
    };

    // Extract fields similar to how we do in save_card / import
    let name = card_data
        .get("name")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let description = card_data
        .get("description")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Author: creator -> creator_notes (matching cards.rs)
    let author = card_data
        .get("creator")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            card_data
                .get("creator_notes")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        });

    let spec = json
        .get("spec")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let spec_version = json
        .get("spec_version")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Tags: Array OR String (comma separated)
    let tags_json = if let Some(tags_value) = card_data.get("tags") {
        if tags_value.is_array() {
            serde_json::to_string_pretty(tags_value).unwrap_or_else(|_| "[]".to_string())
        } else if let Some(tags_str) = tags_value.as_str() {
            let tags: Vec<&str> = tags_str
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            serde_json::to_string_pretty(&tags).unwrap_or_else(|_| "[]".to_string())
        } else {
            "[]".to_string()
        }
    } else {
        "[]".to_string()
    };

    // Update the card record
    if let Some(n) = name {
        card_active.name = Set(n);
    }
    if let Some(d) = description {
        card_active.description = Set(Some(d));
    }
    if let Some(a) = author {
        card_active.author = Set(Some(a));
    }
    card_active.tags = Set(tags_json);
    if let Some(s) = spec {
        card_active.spec = Set(Some(s));
    }
    if let Some(sv) = spec_version {
        card_active.spec_version = Set(Some(sv));
    }

    // Always update the full data blob
    card_active.data = Set(version.data);

    // Explicitly set the card's displayed version to match the restored snapshot
    card_active.version = Set(Some(version.version_number));

    card_active.updated_at = Set(chrono::Utc::now().naive_utc());

    card_active
        .update(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::OK)
}

/// 删除版本
pub async fn delete_version(
    State(db): State<DatabaseConnection>,
    Path((_card_id, version_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, String)> {
    // 不严格检查 character_id 匹配，只要 ID 对即可删除
    let res = CharacterVersion::delete_by_id(version_id)
        .exec(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if res.rows_affected == 0 {
        return Err((StatusCode::NOT_FOUND, "Version not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}
