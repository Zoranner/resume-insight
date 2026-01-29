use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "resumes")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub filename: String,
    pub file_hash: String,
    pub file_url: String,
    pub status: String, // pending | analyzing | completed | failed
    pub job_key: Option<String>,
    pub error_message: Option<String>,
    pub uploaded_at: DateTime,
    pub analyzed_at: Option<DateTime>,
    pub analysis_json: Option<String>,
    pub name: Option<String>,
    pub score: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// 辅助枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResumeStatus {
    Pending,
    Analyzing,
    Completed,
    Failed,
}

impl ResumeStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Pending => "pending",
            Self::Analyzing => "analyzing",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }

    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Self {
        match s {
            "pending" => Self::Pending,
            "analyzing" => Self::Analyzing,
            "completed" => Self::Completed,
            "failed" => Self::Failed,
            _ => Self::Pending,
        }
    }
}
