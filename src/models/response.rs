use serde::Serialize;

use super::Analysis;

/// API 分析响应
#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct AnalysisResponse {
    pub filename: String,
    pub job_key: Option<String>,
    pub analysis: Analysis,
}
