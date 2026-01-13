use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CharacterVersions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(CharacterVersions::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(CharacterVersions::CharacterId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CharacterVersions::VersionNumber)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(CharacterVersions::Note).string())
                    .col(ColumnDef::new(CharacterVersions::Data).string().not_null()) // JSON snapshot
                    .col(
                        ColumnDef::new(CharacterVersions::CreatedAt)
                            .timestamp()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_character_versions_character_id")
                            .from(CharacterVersions::Table, CharacterVersions::CharacterId)
                            .to(CharacterCards::Table, CharacterCards::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CharacterVersions::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum CharacterVersions {
    Table,
    Id,
    CharacterId,
    VersionNumber,
    Note,
    Data,
    CreatedAt,
}

#[derive(Iden)]
pub enum CharacterCards {
    Table,
    Id,
}
