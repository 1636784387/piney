//! `SeaORM` Entity - QuickReply
//!
//! 快速回复实体，与角色卡关联

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "quick_replies")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub card_id: Uuid,
    pub file_name: String,
    pub display_name: String,
    pub file_size: i64,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::character_card::Entity",
        from = "Column::CardId",
        to = "super::character_card::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    CharacterCard,
}

impl Related<super::character_card::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CharacterCard.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
