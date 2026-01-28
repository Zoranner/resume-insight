use anyhow::Result;
use serde::Serialize;
use std::path::PathBuf;
use tokio::fs;

/// 日志记录器
pub struct Logger {
    log_dir: PathBuf,
}

impl Logger {
    /// 创建日志记录器
    pub fn new(log_dir: impl Into<PathBuf>) -> Self {
        Self {
            log_dir: log_dir.into(),
        }
    }

    /// 确保日志目录存在
    pub async fn ensure_log_dir(&self) -> Result<()> {
        if !self.log_dir.exists() {
            fs::create_dir_all(&self.log_dir).await?;
            tracing::info!("Log directory created: {}", self.log_dir.display());
        }
        Ok(())
    }

    /// 记录 LLM 请求日志
    pub async fn log_llm_request<T: Serialize>(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        file_url: &str,
        request: &T,
    ) -> Result<()> {
        self.ensure_log_dir().await?;

        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S%.3f");
        let log_file = self.log_dir.join(format!("llm_request_{}.log", timestamp));

        let request_json = serde_json::to_string_pretty(request)
            .unwrap_or_else(|e| format!("Failed to serialize request: {}", e));

        let log_content = format!(
            r#"================================================================================
LLM Request Log
================================================================================
Timestamp: {}
File URL: {}

================================================================================
System Prompt
================================================================================
{}

================================================================================
User Prompt
================================================================================
{}

================================================================================
Request Payload (JSON)
================================================================================
{}
"#,
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
            file_url,
            system_prompt,
            user_prompt,
            request_json
        );

        fs::write(&log_file, log_content).await?;
        tracing::debug!("LLM request logged to: {}", log_file.display());

        Ok(())
    }

    /// 记录 LLM 响应日志
    pub async fn log_llm_response(&self, content: &str) -> Result<()> {
        self.ensure_log_dir().await?;

        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S%.3f");
        let log_file = self.log_dir.join(format!("llm_response_{}.log", timestamp));

        let log_content = format!(
            r#"================================================================================
LLM Response Log
================================================================================
Timestamp: {}
Content Length: {} chars
Content Lines: {}

================================================================================
Response Content
================================================================================
{}
"#,
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
            content.len(),
            content.lines().count(),
            content
        );

        fs::write(&log_file, log_content).await?;
        tracing::debug!("LLM response logged to: {}", log_file.display());

        Ok(())
    }

    /// 记录错误日志
    pub async fn log_error(&self, context: &str, error: &str) -> Result<()> {
        self.ensure_log_dir().await?;

        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S%.3f");
        let log_file = self.log_dir.join(format!("error_{}.log", timestamp));

        let log_content = format!(
            r#"================================================================================
Error Log
================================================================================
Timestamp: {}
Context: {}

================================================================================
Error Details
================================================================================
{}
"#,
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
            context,
            error
        );

        fs::write(&log_file, log_content).await?;
        tracing::debug!("Error logged to: {}", log_file.display());

        Ok(())
    }
}
