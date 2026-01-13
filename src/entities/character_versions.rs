use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "character_versions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub character_id: Uuid,
    pub version_number: String,
    pub note: Option<String>,
    #[sea_orm(column_type = "Text")]
    pub data: String,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::character_card::Entity",
        from = "Column::CharacterId",
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
