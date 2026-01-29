use sea_orm::*;
use chrono::Utc;
use crate::entities::{resume, prelude::*};
use crate::models::Analysis;

#[derive(Debug, Clone)]
pub struct ListFilters {
    pub status: Option<String>,
    pub job_key: Option<String>,
    pub search: Option<String>,
    pub page: u64,
    pub page_size: u64,
}

impl Default for ListFilters {
    fn default() -> Self {
        Self {
            status: None,
            job_key: None,
            search: None,
            page: 1,
            page_size: 20,
        }
    }
}

pub struct ResumeRepository {
    db: DatabaseConnection,
}

impl ResumeRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// 创建简历记录
    pub async fn create(&self, model: resume::ActiveModel) -> Result<resume::Model, DbErr> {
        model.insert(&self.db).await
    }

    /// 根据 ID 查找简历
    pub async fn find_by_id(&self, id: &str) -> Result<Option<resume::Model>, DbErr> {
        Resume::find_by_id(id).one(&self.db).await
    }

    /// 根据文件哈希查找简历（去重）
    pub async fn find_by_hash(&self, hash: &str) -> Result<Option<resume::Model>, DbErr> {
        Resume::find()
            .filter(resume::Column::FileHash.eq(hash))
            .one(&self.db)
            .await
    }

    /// 列表查询（支持分页和筛选）
    pub async fn list(&self, filters: ListFilters) -> Result<(Vec<resume::Model>, u64), DbErr> {
        let mut query = Resume::find();

        // 状态筛选
        if let Some(status) = &filters.status {
            query = query.filter(resume::Column::Status.eq(status));
        }

        // 岗位筛选
        if let Some(job_key) = &filters.job_key {
            query = query.filter(resume::Column::JobKey.eq(job_key));
        }

        // 搜索（姓名或文件名）
        if let Some(search) = &filters.search {
            let search_pattern = format!("%{}%", search);
            query = query.filter(
                Condition::any()
                    .add(resume::Column::Name.like(&search_pattern))
                    .add(resume::Column::Filename.like(&search_pattern)),
            );
        }

        // 排序：最新上传的在前
        query = query.order_by_desc(resume::Column::UploadedAt);

        // 统计总数
        let total = query.clone().count(&self.db).await?;

        // 分页
        let paginator = query.paginate(&self.db, filters.page_size);
        let items = paginator.fetch_page(filters.page - 1).await?;

        Ok((items, total))
    }

    /// 更新状态
    pub async fn update_status(
        &self,
        id: &str,
        status: &str,
        error_message: Option<String>,
    ) -> Result<(), DbErr> {
        let mut update: resume::ActiveModel = Resume::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or(DbErr::RecordNotFound(format!("Resume {} not found", id)))?
            .into();

        update.status = Set(status.to_string());
        
        if status == "analyzing" || status == "completed" {
            update.analyzed_at = Set(Some(Utc::now().naive_utc()));
        }
        
        if let Some(msg) = error_message {
            update.error_message = Set(Some(msg));
        }

        update.update(&self.db).await?;
        Ok(())
    }

    /// 保存分析结果
    pub async fn save_analysis(&self, id: &str, analysis: &Analysis) -> Result<(), DbErr> {
        let mut update: resume::ActiveModel = Resume::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or(DbErr::RecordNotFound(format!("Resume {} not found", id)))?
            .into();

        // 序列化分析结果
        let analysis_json = serde_json::to_string(analysis)
            .map_err(|e| DbErr::Custom(format!("Failed to serialize analysis: {}", e)))?;

        update.analysis_json = Set(Some(analysis_json));
        update.status = Set("completed".to_string());
        update.analyzed_at = Set(Some(Utc::now().naive_utc()));
        update.name = Set(Some(analysis.basic_info.name.clone()));
        update.score = Set(Some(analysis.score as i32));

        update.update(&self.db).await?;
        Ok(())
    }

    /// 删除简历
    pub async fn delete(&self, id: &str) -> Result<(), DbErr> {
        Resume::delete_by_id(id).exec(&self.db).await?;
        Ok(())
    }

    /// 批量更新状态
    pub async fn batch_update_status(&self, ids: Vec<String>, status: &str) -> Result<(), DbErr> {
        Resume::update_many()
            .col_expr(resume::Column::Status, sea_orm::sea_query::Expr::value(status))
            .filter(resume::Column::Id.is_in(ids))
            .exec(&self.db)
            .await?;
        Ok(())
    }
}
