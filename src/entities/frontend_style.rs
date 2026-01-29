//! `SeaORM` Entity - 前端样式

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "frontend_style")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    #[sea_orm(column_type = "Text")]
    pub original_text: String,
    #[sea_orm(column_type = "Text")]
    pub regex_pattern: String,
    #[sea_orm(column_type = "Text")]
    pub html_code: String,
    #[sea_orm(column_type = "Text")]
    pub worldinfo_key: String,
    #[sea_orm(column_type = "Text")]
    pub worldinfo_content: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
