//! 数据库初始化脚本 v1.0.0
//!
//! 合并了所有开发阶段的迁移脚本，创建完整的数据库结构。
//! 所有表使用 if_not_exists，确保老用户也能获得缺失的新表。

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 检测是否为老用户（数据库已存在）
        let is_existing_db = manager.has_table("character_cards").await?;

        if is_existing_db {
            println!("🔄 检测到现有数据库，将检查并创建缺失的表...");
        } else {
            println!("🆕 首次初始化，创建数据库结构...");
        }

        // 创建所有表（使用 if_not_exists，已存在的表会安全跳过）

        // ==================== settings ====================
        manager
            .create_table(
                Table::create()
                    .table(Settings::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Settings::Key)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Settings::Value).text().not_null())
                    .col(ColumnDef::new(Settings::UpdatedAt).timestamp().not_null())
                    .to_owned(),
            )
            .await?;

        // ==================== categories ====================
        manager
            .create_table(
                Table::create()
                    .table(Categories::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Categories::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Categories::Name).string().not_null())
                    .col(
                        ColumnDef::new(Categories::SortOrder)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(Categories::CreatedAt).date_time().not_null())
                    .to_owned(),
            )
            .await?;

        // ==================== character_cards ====================
        manager
            .create_table(
                Table::create()
                    .table(CharacterCards::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(CharacterCards::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(CharacterCards::Name).string().not_null())
                    .col(ColumnDef::new(CharacterCards::Description).string())
                    .col(ColumnDef::new(CharacterCards::Author).string())
                    .col(ColumnDef::new(CharacterCards::Avatar).string())
                    .col(ColumnDef::new(CharacterCards::Spec).string())
                    .col(ColumnDef::new(CharacterCards::SpecVersion).string())
                    .col(ColumnDef::new(CharacterCards::Data).text().not_null())
                    .col(
                        ColumnDef::new(CharacterCards::CreatedAt)
                            .date_time()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CharacterCards::UpdatedAt)
                            .date_time()
                            .not_null(),
                    )
                    .col(ColumnDef::new(CharacterCards::CategoryId).uuid())
                    .col(
                        ColumnDef::new(CharacterCards::Tags)
                            .text()
                            .not_null()
                            .default("[]"),
                    )
                    .col(
                        ColumnDef::new(CharacterCards::Rating)
                            .double()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(CharacterCards::CoverBlur)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(CharacterCards::Version).string())
                    .col(ColumnDef::new(CharacterCards::DeletedAt).date_time())
                    .col(ColumnDef::new(CharacterCards::CustomSummary).text())
                    .col(ColumnDef::new(CharacterCards::UserNote).text())
                    .col(
                        ColumnDef::new(CharacterCards::MetadataModified)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(CharacterCards::DataHash).text())
                    .col(ColumnDef::new(CharacterCards::TokenCountTotal).integer())
                    .col(ColumnDef::new(CharacterCards::TokenCountSpec).integer())
                    .col(ColumnDef::new(CharacterCards::TokenCountWb).integer())
                    .col(ColumnDef::new(CharacterCards::TokenCountOther).integer())
                    .col(
                        ColumnDef::new(CharacterCards::Source)
                            .string()
                            .not_null()
                            .default("import"),
                    )
                    .to_owned(),
            )
            .await?;

        // character_cards 索引
        manager
            .create_index(
                Index::create().if_not_exists()
                    .name("idx_character_cards_data_hash")
                    .table(CharacterCards::Table)
                    .col(CharacterCards::DataHash)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create().if_not_exists()
                    .name("idx_character_cards_token_sum")
                    .table(CharacterCards::Table)
                    .col(CharacterCards::DeletedAt)
                    .col(CharacterCards::TokenCountTotal)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create().if_not_exists()
                    .name("idx_character_cards_updated")
                    .table(CharacterCards::Table)
                    .col(CharacterCards::DeletedAt)
                    .col(CharacterCards::UpdatedAt)
                    .to_owned(),
            )
            .await?;

        // ==================== ai_channels ====================
        manager
            .create_table(
                Table::create()
                    .table(AiChannels::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AiChannels::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AiChannels::Name).string().not_null())
                    .col(ColumnDef::new(AiChannels::BaseUrl).string().not_null())
                    .col(ColumnDef::new(AiChannels::ApiKey).string().not_null())
                    .col(ColumnDef::new(AiChannels::ModelId).string().not_null())
                    .col(
                        ColumnDef::new(AiChannels::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(AiChannels::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(AiChannels::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // ==================== world_info ====================
        manager
            .create_table(
                Table::create()
                    .table(WorldInfo::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(WorldInfo::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(WorldInfo::Name).string().not_null())
                    .col(ColumnDef::new(WorldInfo::Data).text().not_null())
                    .col(ColumnDef::new(WorldInfo::CreatedAt).date_time().not_null())
                    .col(ColumnDef::new(WorldInfo::UpdatedAt).date_time().not_null())
                    .to_owned(),
            )
            .await?;

        // ==================== chat_histories ====================
        manager
            .create_table(
                Table::create()
                    .table(ChatHistories::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ChatHistories::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ChatHistories::CardId).uuid().not_null())
                    .col(ColumnDef::new(ChatHistories::FileName).string().not_null())
                    .col(
                        ColumnDef::new(ChatHistories::DisplayName)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ChatHistories::FileSize)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(ChatHistories::Format).string().not_null())
                    .col(
                        ColumnDef::new(ChatHistories::Progress)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(ChatHistories::CreatedAt)
                            .date_time()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ChatHistories::UpdatedAt)
                            .date_time()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ChatHistories::SourceFileName).string())
                    .col(
                        ColumnDef::new(ChatHistories::CurrentPage)
                            .integer()
                            .not_null()
                            .default(1),
                    )
                    .col(ColumnDef::new(ChatHistories::ReadingSettings).string())
                    .col(
                        ColumnDef::new(ChatHistories::RegexScripts)
                            .text()
                            .not_null()
                            .default("[]"),
                    )
                    .to_owned(),
            )
            .await?;

        // ==================== character_versions ====================
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
                    .col(ColumnDef::new(CharacterVersions::Data).string().not_null())
                    .col(
                        ColumnDef::new(CharacterVersions::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(CharacterVersions::Table, CharacterVersions::CharacterId)
                            .to(CharacterCards::Table, CharacterCards::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // ==================== doctor_task ====================
        manager
            .create_table(
                Table::create()
                    .table(DoctorTask::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(DoctorTask::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(DoctorTask::CharacterId).uuid().not_null())
                    .col(
                        ColumnDef::new(DoctorTask::Status)
                            .string()
                            .not_null()
                            .default("pending"),
                    )
                    .col(ColumnDef::new(DoctorTask::FinalReport).text())
                    .col(ColumnDef::new(DoctorTask::CreatedAt).date_time().not_null())
                    .col(ColumnDef::new(DoctorTask::UpdatedAt).date_time().not_null())
                    .to_owned(),
            )
            .await?;

        // ==================== quick_replies ====================
        manager
            .create_table(
                Table::create()
                    .table(QuickReplies::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(QuickReplies::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(QuickReplies::CardId).uuid().not_null())
                    .col(ColumnDef::new(QuickReplies::FileName).string().not_null())
                    .col(
                        ColumnDef::new(QuickReplies::DisplayName)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(QuickReplies::FileSize)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(QuickReplies::CreatedAt)
                            .date_time()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(QuickReplies::UpdatedAt)
                            .date_time()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(QuickReplies::Table, QuickReplies::CardId)
                            .to(CharacterCards::Table, CharacterCards::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create().if_not_exists()
                    .name("idx_quick_replies_card_id")
                    .table(QuickReplies::Table)
                    .col(QuickReplies::CardId)
                    .to_owned(),
            )
            .await?;

        // ==================== theaters ====================
        manager
            .create_table(
                Table::create()
                    .table(Theaters::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Theaters::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Theaters::Title).string().not_null())
                    .col(
                        ColumnDef::new(Theaters::Category)
                            .string()
                            .not_null()
                            .default("未分类"),
                    )
                    .col(ColumnDef::new(Theaters::Desc).text().not_null())
                    .col(ColumnDef::new(Theaters::Content).text().not_null())
                    .col(ColumnDef::new(Theaters::CreatedAt).date_time().not_null())
                    .col(ColumnDef::new(Theaters::UpdatedAt).date_time().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create().if_not_exists()
                    .name("idx_theaters_category")
                    .table(Theaters::Table)
                    .col(Theaters::Category)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create().if_not_exists()
                    .name("idx_theaters_sort")
                    .table(Theaters::Table)
                    .col(Theaters::UpdatedAt)
                    .col(Theaters::CreatedAt)
                    .to_owned(),
            )
            .await?;

        // ==================== frontend_style ====================
        manager
            .create_table(
                Table::create()
                    .table(FrontendStyle::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(FrontendStyle::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(FrontendStyle::Name).string().not_null())
                    .col(
                        ColumnDef::new(FrontendStyle::OriginalText)
                            .text()
                            .not_null()
                            .default(""),
                    )
                    .col(
                        ColumnDef::new(FrontendStyle::RegexPattern)
                            .text()
                            .not_null()
                            .default(""),
                    )
                    .col(
                        ColumnDef::new(FrontendStyle::HtmlCode)
                            .text()
                            .not_null()
                            .default(""),
                    )
                    .col(
                        ColumnDef::new(FrontendStyle::WorldinfoKey)
                            .text()
                            .not_null()
                            .default(""),
                    )
                    .col(
                        ColumnDef::new(FrontendStyle::WorldinfoContent)
                            .text()
                            .not_null()
                            .default(""),
                    )
                    .col(
                        ColumnDef::new(FrontendStyle::CreatedAt)
                            .date_time()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(FrontendStyle::UpdatedAt)
                            .date_time()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create().if_not_exists()
                    .name("idx_frontend_style_name")
                    .table(FrontendStyle::Table)
                    .col(FrontendStyle::Name)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create().if_not_exists()
                    .name("idx_frontend_style_sort")
                    .table(FrontendStyle::Table)
                    .col(FrontendStyle::UpdatedAt)
                    .to_owned(),
            )
            .await?;

        // ==================== image_category ====================
        manager
            .create_table(
                Table::create()
                    .table(ImageCategory::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ImageCategory::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ImageCategory::Name)
                            .string_len(100)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ImageCategory::SortOrder)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(ImageCategory::CreatedAt)
                            .date_time()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create().if_not_exists()
                    .name("idx_image_category_sort")
                    .table(ImageCategory::Table)
                    .col(ImageCategory::SortOrder)
                    .to_owned(),
            )
            .await?;

        // ==================== image ====================
        manager
            .create_table(
                Table::create()
                    .table(Image::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Image::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Image::Title).string_len(255).not_null())
                    .col(ColumnDef::new(Image::CategoryId).uuid())
                    .col(ColumnDef::new(Image::Tags).text().not_null().default("[]"))
                    .col(ColumnDef::new(Image::FilePath).text().not_null())
                    .col(ColumnDef::new(Image::ThumbnailPath).text().not_null())
                    .col(ColumnDef::new(Image::Width).integer().not_null())
                    .col(ColumnDef::new(Image::Height).integer().not_null())
                    .col(ColumnDef::new(Image::FileSize).big_integer().not_null())
                    .col(ColumnDef::new(Image::ColorCategory).string_len(10))
                    .col(
                        ColumnDef::new(Image::IsAi)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(Image::AiPlatform).string_len(20))
                    .col(ColumnDef::new(Image::AiPrompt).text())
                    .col(ColumnDef::new(Image::AiNegativePrompt).text())
                    .col(
                        ColumnDef::new(Image::IsAuthorized)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Image::IsFavorite)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(Image::UserNotes).text())
                    .col(
                        ColumnDef::new(Image::CharCards)
                            .text()
                            .not_null()
                            .default("[]"),
                    )
                    .col(ColumnDef::new(Image::CreatedAt).date_time().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Image::Table, Image::CategoryId)
                            .to(ImageCategory::Table, ImageCategory::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create().if_not_exists()
                    .name("idx_image_category_id")
                    .table(Image::Table)
                    .col(Image::CategoryId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create().if_not_exists()
                    .name("idx_image_favorite")
                    .table(Image::Table)
                    .col(Image::IsFavorite)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create().if_not_exists()
                    .name("idx_image_color")
                    .table(Image::Table)
                    .col(Image::ColorCategory)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create().if_not_exists()
                    .name("idx_image_created_at")
                    .table(Image::Table)
                    .col(Image::CreatedAt)
                    .to_owned(),
            )
            .await?;

        println!("✅ 数据库初始化完成！");
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 按依赖顺序删除表（先删子表，再删父表）
        manager
            .drop_table(Table::drop().table(Image::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ImageCategory::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(FrontendStyle::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Theaters::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(QuickReplies::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(DoctorTask::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(CharacterVersions::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ChatHistories::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(WorldInfo::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(AiChannels::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(CharacterCards::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Categories::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Settings::Table).to_owned())
            .await?;
        Ok(())
    }
}

// ==================== 表定义枚举 ====================

#[derive(DeriveIden)]
enum Settings {
    Table,
    Key,
    Value,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Categories {
    Table,
    Id,
    Name,
    SortOrder,
    CreatedAt,
}

#[derive(DeriveIden)]
enum CharacterCards {
    Table,
    Id,
    Name,
    Description,
    Author,
    Avatar,
    Spec,
    SpecVersion,
    Data,
    CreatedAt,
    UpdatedAt,
    CategoryId,
    Tags,
    Rating,
    CoverBlur,
    Version,
    DeletedAt,
    CustomSummary,
    UserNote,
    MetadataModified,
    DataHash,
    TokenCountTotal,
    TokenCountSpec,
    TokenCountWb,
    TokenCountOther,
    Source,
}

#[derive(DeriveIden)]
enum AiChannels {
    Table,
    Id,
    Name,
    BaseUrl,
    ApiKey,
    ModelId,
    IsActive,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum WorldInfo {
    Table,
    Id,
    Name,
    Data,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum ChatHistories {
    Table,
    Id,
    CardId,
    FileName,
    DisplayName,
    FileSize,
    Format,
    Progress,
    CreatedAt,
    UpdatedAt,
    SourceFileName,
    CurrentPage,
    ReadingSettings,
    RegexScripts,
}

#[derive(DeriveIden)]
enum CharacterVersions {
    Table,
    Id,
    CharacterId,
    VersionNumber,
    Note,
    Data,
    CreatedAt,
}

#[derive(DeriveIden)]
enum DoctorTask {
    Table,
    Id,
    CharacterId,
    Status,
    FinalReport,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum QuickReplies {
    Table,
    Id,
    CardId,
    FileName,
    DisplayName,
    FileSize,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Theaters {
    Table,
    Id,
    Title,
    Category,
    Desc,
    Content,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum FrontendStyle {
    Table,
    Id,
    Name,
    OriginalText,
    RegexPattern,
    HtmlCode,
    WorldinfoKey,
    WorldinfoContent,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum ImageCategory {
    Table,
    Id,
    Name,
    SortOrder,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Image {
    Table,
    Id,
    Title,
    CategoryId,
    Tags,
    FilePath,
    ThumbnailPath,
    Width,
    Height,
    FileSize,
    ColorCategory,
    IsAi,
    AiPlatform,
    AiPrompt,
    AiNegativePrompt,
    IsAuthorized,
    IsFavorite,
    UserNotes,
    CharCards,
    CreatedAt,
}
