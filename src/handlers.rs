use axum::{
    extract::{Multipart, Path, Query, State},
    Json,
};
use chrono::Utc;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    config::Config,
    entities::{resume, ResumeStatus},
    error::AppError,
    models::Analysis,
    repositories::{ListFilters, ResumeRepository},
    services::Analyzer,
};

#[derive(Clone)]
pub struct AppState {
    pub analyzer: Arc<Analyzer>,
    pub repo: Arc<ResumeRepository>,
}

impl AppState {
    pub fn new(config: Config, db: DatabaseConnection) -> Result<Self, anyhow::Error> {
        Ok(Self {
            analyzer: Arc::new(Analyzer::new(config.llm, config.server)?),
            repo: Arc::new(ResumeRepository::new(db)),
        })
    }
}

/// 健康检查
pub async fn health_check() -> &'static str {
    "OK"
}

// ============================================================================
// 上传接口
// ============================================================================

#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub uploaded: Vec<UploadedFile>,
}

#[derive(Debug, Serialize)]
pub struct UploadedFile {
    pub id: String,
    pub filename: String,
    pub status: String,
}

/// 上传简历文件（不分析）
pub async fn upload_resumes(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, AppError> {
    tracing::info!("Received upload request");

    let mut uploaded_files = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::FileError(format!("Failed to read field: {}", e)))?
    {
        if field.name().unwrap_or("") == "file" {
            let filename = field.file_name().unwrap_or("unknown").to_string();
            let data = field
                .bytes()
                .await
                .map_err(|e| AppError::FileError(format!("Failed to read bytes: {}", e)))?
                .to_vec();

            tracing::info!("Processing file: {} ({} bytes)", filename, data.len());

            // 保存文件并创建数据库记录
            let file_url = state.analyzer.save_file(&data, &filename).await?;
            let file_hash = state.analyzer.calculate_hash(&data);

            // 检查是否已存在
            if let Some(existing) = state.repo.find_by_hash(&file_hash).await.map_err(|e| {
                AppError::Internal(anyhow::anyhow!("Database error: {}", e))
            })? {
                tracing::info!("File already exists: {}", existing.id);
                uploaded_files.push(UploadedFile {
                    id: existing.id,
                    filename: existing.filename,
                    status: existing.status,
                });
                continue;
            }

            // 创建新记录
            let id = Uuid::new_v4().to_string();
            let resume = resume::ActiveModel {
                id: sea_orm::Set(id.clone()),
                filename: sea_orm::Set(filename.clone()),
                file_hash: sea_orm::Set(file_hash),
                file_url: sea_orm::Set(file_url),
                status: sea_orm::Set(ResumeStatus::Pending.as_str().to_string()),
                job_key: sea_orm::Set(None),
                error_message: sea_orm::Set(None),
                uploaded_at: sea_orm::Set(Utc::now().naive_utc()),
                analyzed_at: sea_orm::Set(None),
                analysis_json: sea_orm::Set(None),
                name: sea_orm::Set(None),
                score: sea_orm::Set(None),
            };

            state.repo.create(resume).await.map_err(|e| {
                AppError::Internal(anyhow::anyhow!("Failed to create resume record: {}", e))
            })?;

            tracing::info!("Created resume record: {}", id);

            uploaded_files.push(UploadedFile {
                id,
                filename,
                status: ResumeStatus::Pending.as_str().to_string(),
            });
        }
    }

    Ok(Json(UploadResponse {
        uploaded: uploaded_files,
    }))
}

// ============================================================================
// 分析接口
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AnalyzeRequest {
    pub resume_ids: Vec<String>,
    pub job: String,
}

#[derive(Debug, Serialize)]
pub struct AnalyzeResponse {
    pub message: String,
    pub count: usize,
}

/// 触发分析（可批量）
pub async fn analyze_resumes(
    State(state): State<AppState>,
    Json(req): Json<AnalyzeRequest>,
) -> Result<Json<AnalyzeResponse>, AppError> {
    tracing::info!("Received analyze request for {} resumes", req.resume_ids.len());

    // 更新状态为 analyzing
    state
        .repo
        .batch_update_status(req.resume_ids.clone(), ResumeStatus::Analyzing.as_str())
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to update status: {}", e)))?;

    // 异步分析每个简历
    for resume_id in &req.resume_ids {
        let state = state.clone();
        let resume_id = resume_id.clone();
        let job_key = req.job.clone();

        // 在后台任务中分析
        tokio::spawn(async move {
            if let Err(e) = analyze_single_resume(state, resume_id, job_key).await {
                tracing::error!("Failed to analyze resume: {}", e);
            }
        });
    }

    Ok(Json(AnalyzeResponse {
        message: "开始分析".to_string(),
        count: req.resume_ids.len(),
    }))
}

/// 分析单个简历（内部函数）
async fn analyze_single_resume(
    state: AppState,
    resume_id: String,
    job_key: String,
) -> Result<(), AppError> {
    tracing::info!("Analyzing resume: {}", resume_id);

    // 获取简历记录
    let resume = state
        .repo
        .find_by_id(&resume_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Database error: {}", e)))?
        .ok_or_else(|| AppError::FileError(format!("Resume {} not found", resume_id)))?;

    // 从 file_url 读取文件
    let file_data = tokio::fs::read(&resume.file_url.replace("http://localhost:3000/files/", "data/files/"))
        .await
        .map_err(|e| AppError::FileError(format!("Failed to read file: {}", e)))?;

    // 调用分析服务
    match state
        .analyzer
        .analyze_file(&file_data, &resume.filename, Some(&job_key))
        .await
    {
        Ok(analysis) => {
            // 保存分析结果
            state
                .repo
                .save_analysis(&resume_id, &analysis)
                .await
                .map_err(|e| {
                    AppError::Internal(anyhow::anyhow!("Failed to save analysis: {}", e))
                })?;

            tracing::info!("Analysis completed for resume: {}", resume_id);
        }
        Err(e) => {
            // 更新为失败状态
            state
                .repo
                .update_status(
                    &resume_id,
                    ResumeStatus::Failed.as_str(),
                    Some(e.to_string()),
                )
                .await
                .map_err(|e| {
                    AppError::Internal(anyhow::anyhow!("Failed to update status: {}", e))
                })?;

            tracing::error!("Analysis failed for resume {}: {}", resume_id, e);
            return Err(e);
        }
    }

    Ok(())
}

// ============================================================================
// 查询接口
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub status: Option<String>,
    pub job_key: Option<String>,
    pub search: Option<String>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct ListResponse {
    pub total: u64,
    pub items: Vec<ResumeListItem>,
}

#[derive(Debug, Serialize)]
pub struct ResumeListItem {
    pub id: String,
    pub filename: String,
    pub status: String,
    pub job_key: Option<String>,
    pub score: Option<i32>,
    pub name: Option<String>,
    pub uploaded_at: String,
    pub analyzed_at: Option<String>,
}

/// 查询简历列表
pub async fn list_resumes(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<ListResponse>, AppError> {
    tracing::debug!("List resumes query: {:?}", query);

    let filters = ListFilters {
        status: query.status,
        job_key: query.job_key,
        search: query.search,
        page: query.page.unwrap_or(1),
        page_size: query.page_size.unwrap_or(20),
    };

    let (items, total) = state
        .repo
        .list(filters)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Database error: {}", e)))?;

    let items = items
        .into_iter()
        .map(|r| ResumeListItem {
            id: r.id,
            filename: r.filename,
            status: r.status,
            job_key: r.job_key,
            score: r.score,
            name: r.name,
            uploaded_at: r.uploaded_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            analyzed_at: r
                .analyzed_at
                .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
        })
        .collect();

    Ok(Json(ListResponse { total, items }))
}

#[derive(Debug, Serialize)]
pub struct ResumeDetail {
    pub id: String,
    pub filename: String,
    pub file_url: String,
    pub status: String,
    pub job_key: Option<String>,
    pub error_message: Option<String>,
    pub uploaded_at: String,
    pub analyzed_at: Option<String>,
    pub analysis: Option<Analysis>,
}

/// 查询简历详情
pub async fn get_resume_detail(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ResumeDetail>, AppError> {
    tracing::debug!("Get resume detail: {}", id);

    let resume = state
        .repo
        .find_by_id(&id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Database error: {}", e)))?
        .ok_or_else(|| AppError::FileError(format!("Resume {} not found", id)))?;

    let analysis = if let Some(json) = &resume.analysis_json {
        serde_json::from_str(json)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to parse analysis: {}", e)))?
    } else {
        None
    };

    Ok(Json(ResumeDetail {
        id: resume.id,
        filename: resume.filename,
        file_url: resume.file_url,
        status: resume.status,
        job_key: resume.job_key,
        error_message: resume.error_message,
        uploaded_at: resume.uploaded_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        analyzed_at: resume
            .analyzed_at
            .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()),
        analysis,
    }))
}

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub status: String,
    pub progress: Option<u8>,
}

/// 查询分析状态
pub async fn get_resume_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<StatusResponse>, AppError> {
    let resume = state
        .repo
        .find_by_id(&id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Database error: {}", e)))?
        .ok_or_else(|| AppError::FileError(format!("Resume {} not found", id)))?;

    // 简单的进度模拟（analyzing 时返回 50%）
    let progress = if resume.status == ResumeStatus::Analyzing.as_str() {
        Some(50)
    } else {
        None
    };

    Ok(Json(StatusResponse {
        status: resume.status,
        progress,
    }))
}

// ============================================================================
// 删除接口
// ============================================================================

#[derive(Debug, Serialize)]
pub struct DeleteResponse {
    pub message: String,
}

/// 删除简历
pub async fn delete_resume(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<DeleteResponse>, AppError> {
    tracing::info!("Deleting resume: {}", id);

    state
        .repo
        .delete(&id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to delete resume: {}", e)))?;

    Ok(Json(DeleteResponse {
        message: "简历已删除".to_string(),
    }))
}
