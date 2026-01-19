use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
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
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(DoctorTask::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum DoctorTask {
    Table,
    Id,
    CharacterId,
    Status,
    FinalReport,
    CreatedAt,
    UpdatedAt,
}
