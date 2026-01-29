use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Resume::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Resume::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Resume::Filename).string().not_null())
                    .col(ColumnDef::new(Resume::FileHash).string().not_null())
                    .col(ColumnDef::new(Resume::FileUrl).string().not_null())
                    .col(ColumnDef::new(Resume::Status).string().not_null())
                    .col(ColumnDef::new(Resume::JobKey).string())
                    .col(ColumnDef::new(Resume::ErrorMessage).text())
                    .col(
                        ColumnDef::new(Resume::UploadedAt)
                            .date_time()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Resume::AnalyzedAt).date_time())
                    .col(ColumnDef::new(Resume::AnalysisJson).text())
                    .col(ColumnDef::new(Resume::Name).string())
                    .col(ColumnDef::new(Resume::Score).integer())
                    .to_owned(),
            )
            .await?;

        // 创建索引
        manager
            .create_index(
                Index::create()
                    .name("idx_resumes_status")
                    .table(Resume::Table)
                    .col(Resume::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_resumes_job_key")
                    .table(Resume::Table)
                    .col(Resume::JobKey)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_resumes_uploaded_at")
                    .table(Resume::Table)
                    .col(Resume::UploadedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_resumes_file_hash")
                    .table(Resume::Table)
                    .col(Resume::FileHash)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Resume::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Resume {
    Table,
    Id,
    Filename,
    FileHash,
    FileUrl,
    Status,
    JobKey,
    ErrorMessage,
    UploadedAt,
    AnalyzedAt,
    AnalysisJson,
    Name,
    Score,
}
