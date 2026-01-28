use axum::{
    extract::{Multipart, State},
    Json,
};
use std::sync::Arc;

use crate::{config::Config, error::AppError, models::AnalysisResponse, services::Analyzer};

#[derive(Clone)]
pub struct AppState {
    pub analyzer: Arc<Analyzer>,
}

impl AppState {
    pub fn new(config: Config) -> Result<Self, anyhow::Error> {
        Ok(Self {
            analyzer: Arc::new(Analyzer::new(config.llm, config.server)?),
        })
    }
}

/// 健康检查
pub async fn health_check() -> &'static str {
    "OK"
}

/// 分析简历
pub async fn analyze_resume(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<AnalysisResponse>, AppError> {
    tracing::info!("Received analyze request");

    // 1. 提取表单数据
    let mut file_data: Option<(String, Vec<u8>)> = None;
    let mut job_key: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::FileError(format!("Failed to read field: {}", e)))?
    {
        match field.name().unwrap_or("") {
            "file" => {
                let filename = field.file_name().unwrap_or("unknown").to_string();
                let data = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::FileError(format!("Failed to read bytes: {}", e)))?
                    .to_vec();
                tracing::info!("Received file: {} ({} bytes)", filename, data.len());
                file_data = Some((filename, data));
            }
            "job" => {
                let job = field
                    .text()
                    .await
                    .map_err(|e| AppError::FileError(format!("Failed to read job: {}", e)))?;
                tracing::info!("Job key: {}", job);
                job_key = Some(job);
            }
            _ => {}
        }
    }

    let (filename, data) =
        file_data.ok_or_else(|| AppError::FileError("No file provided".to_string()))?;

    // 2. 使用视觉模型分析文件
    tracing::info!("Analyzing resume with vision model (job: {:?})...", job_key);
    let analysis = state
        .analyzer
        .analyze_file(&data, &filename, job_key.as_deref())
        .await?;
    tracing::info!("Analysis completed: score={}", analysis.score);

    Ok(Json(AnalysisResponse {
        filename,
        job_key,
        analysis,
    }))
}
