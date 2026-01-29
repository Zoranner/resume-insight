use anyhow::{Context, Result};
use reqwest::Client;
use sha2::{Digest, Sha256};
use std::{path::PathBuf, sync::Arc};
use tokio::fs;

use crate::{
    config::{LlmConfig, ServerConfig},
    error::AppError,
    logger::Logger,
    models::{
        Analysis, ChatRequest, ChatResponse, ContentPart, Message, MessageContent, ThinkingConfig,
    },
    prompts::PromptManager,
};

pub struct Analyzer {
    config: LlmConfig,
    server_config: ServerConfig,
    client: Client,
    prompt_manager: Arc<PromptManager>,
    logger: Logger,
}

impl Analyzer {
    pub fn new(config: LlmConfig, server_config: ServerConfig) -> Result<Self> {
        let prompt_manager = PromptManager::load().context("Failed to load prompt manager")?;
        let logger = Logger::new(&server_config.logs_dir);

        Ok(Self {
            config,
            server_config,
            client: Client::new(),
            prompt_manager: Arc::new(prompt_manager),
            logger,
        })
    }

    pub fn calculate_hash(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    fn get_extension(filename: &str) -> String {
        std::path::Path::new(filename)
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_else(|| "pdf".to_string())
    }

    pub async fn save_file(&self, data: &[u8], filename: &str) -> Result<String, AppError> {
        // 创建文件目录
        let files_dir = PathBuf::from(&self.server_config.files_dir);
        fs::create_dir_all(&files_dir)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to create files dir: {}", e)))?;

        let file_hash = self.calculate_hash(data);
        let extension = Self::get_extension(filename);
        let hash_filename = format!("{}.{}", file_hash, extension);
        let file_path = files_dir.join(&hash_filename);

        // 如果文件已存在，直接返回 URL
        if !file_path.exists() {
            fs::write(&file_path, data)
                .await
                .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to write file: {}", e)))?;
            tracing::info!("New file saved (hash: {})", file_hash);
        } else {
            tracing::info!("File already exists (hash: {}), reusing", file_hash);
        }

        Ok(format!(
            "{}/files/{}",
            self.server_config.base_url.trim_end_matches('/'),
            hash_filename
        ))
    }

    pub async fn analyze_file(
        &self,
        file_data: &[u8],
        filename: &str,
        job_key: Option<&str>,
    ) -> Result<Analysis, AppError> {
        let file_url = self.save_file(file_data, filename).await?;
        let prompt = self
            .prompt_manager
            .build_analysis_prompt_for_vision(job_key)
            .map_err(AppError::Internal)?;

        let system_prompt = self.prompt_manager.get_system_prompt().to_string();

        let request = ChatRequest {
            model: self.config.model.clone(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: Some(MessageContent::Text(system_prompt.clone())),
                },
                Message {
                    role: "user".to_string(),
                    content: Some(MessageContent::MultiModal(vec![
                        ContentPart::FileUrl {
                            file_url: crate::models::FileUrl {
                                url: file_url.clone(),
                            },
                        },
                        ContentPart::Text {
                            text: prompt.clone(),
                        },
                    ])),
                },
            ],
            temperature: Some(0.7),
            thinking: Some(ThinkingConfig {
                thinking_type: "enabled".to_string(),
            }),
        };

        // 📝 记录请求信息
        tracing::info!("🚀 Sending LLM request");
        tracing::debug!("Model: {}", request.model);
        tracing::debug!("File URL: {}", file_url);

        if let Err(e) = self
            .logger
            .log_llm_request(&system_prompt, &prompt, &file_url, &request)
            .await
        {
            tracing::warn!("Failed to write request log: {}", e);
        }

        let url = format!("{}/chat/completions", self.config.base_url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to LLM")
            .map_err(|e| AppError::LlmError(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            let error_msg = format!("LLM API returned status {}: {}", status, error_text);

            // 记录错误日志
            if let Err(e) = self.logger.log_error("LLM API Error", &error_msg).await {
                tracing::warn!("Failed to write error log: {}", e);
            }

            return Err(AppError::LlmError(error_msg));
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .context("Failed to parse LLM response")
            .map_err(|e| AppError::LlmError(e.to_string()))?;

        let content = chat_response
            .choices
            .first()
            .ok_or_else(|| AppError::LlmError("No choices in response".to_string()))?
            .message
            .content
            .clone();

        // 📝 记录响应内容
        tracing::info!("✅ Received LLM response ({} chars)", content.len());

        if let Err(e) = self.logger.log_llm_response(&content).await {
            tracing::warn!("Failed to write response log: {}", e);
        }

        // 解析响应并记录错误
        match self.parse_analysis(&content) {
            Ok(analysis) => Ok(analysis),
            Err(e) => {
                // 记录解析错误
                let error_detail = format!("Parse error: {}\n\nResponse content:\n{}", e, content);
                if let Err(log_err) = self
                    .logger
                    .log_error("XML Parse Error", &error_detail)
                    .await
                {
                    tracing::warn!("Failed to write parse error log: {}", log_err);
                }
                Err(e)
            }
        }
    }

    fn parse_analysis(&self, content: &str) -> Result<Analysis, AppError> {
        // 尝试提取 XML（可能被包裹在 ```xml 代码块中）
        let xml_str = if content.contains("```xml") {
            content
                .split("```xml")
                .nth(1)
                .and_then(|s| s.split("```").next())
                .unwrap_or(content)
        } else if content.contains("```") {
            content
                .split("```")
                .nth(1)
                .and_then(|s| s.split("```").next())
                .unwrap_or(content)
        } else {
            content
        }
        .trim();

        // 提取 <analysis> 标签内容
        let xml_content = if let Some(start) = xml_str.find("<analysis>") {
            if let Some(end) = xml_str.find("</analysis>") {
                &xml_str[start..=end + 10]
            } else {
                xml_str
            }
        } else {
            xml_str
        };

        Self::parse_xml(xml_content)
            .context("Failed to parse analysis XML")
            .map_err(|e| {
                tracing::error!("Failed to parse XML: {}", e);
                tracing::error!("Content: {}", xml_content);
                AppError::LlmError(format!("Failed to parse analysis: {}", e))
            })
    }

    fn parse_xml(xml: &str) -> Result<Analysis> {
        use crate::models::{BasicInfo, Experience, Skills};
        use quick_xml::de::from_str;

        #[derive(Debug, serde::Deserialize)]
        struct XmlAnalysis {
            basic_info: XmlBasicInfo,
            score: u32,
            summary: String,
            skills: XmlSkills,
            experience: XmlExperience,
            strengths: XmlList<String>,
            concerns: XmlList<String>,
            focus: XmlList<String>,
        }

        #[derive(Debug, serde::Deserialize)]
        struct XmlBasicInfo {
            name: String,
            gender: String,
            age: String,
            phone: String,
            email: String,
            location: String,
            work_years: String,
            degree: String,
            major: String,
            school: String,
            current_company: String,
            current_position: String,
        }

        #[derive(Debug, serde::Deserialize)]
        struct XmlSkills {
            level: String,
            details: String,
        }

        #[derive(Debug, serde::Deserialize)]
        struct XmlExperience {
            level: String,
            details: String,
        }

        #[derive(Debug, serde::Deserialize)]
        struct XmlList<T> {
            #[serde(rename = "item", default)]
            items: Vec<T>,
        }

        let xml_analysis: XmlAnalysis = from_str(xml)?;

        Ok(Analysis {
            basic_info: BasicInfo {
                name: xml_analysis.basic_info.name,
                gender: xml_analysis.basic_info.gender,
                age: xml_analysis.basic_info.age,
                phone: xml_analysis.basic_info.phone,
                email: xml_analysis.basic_info.email,
                location: xml_analysis.basic_info.location,
                work_years: xml_analysis.basic_info.work_years,
                degree: xml_analysis.basic_info.degree,
                major: xml_analysis.basic_info.major,
                school: xml_analysis.basic_info.school,
                current_company: xml_analysis.basic_info.current_company,
                current_position: xml_analysis.basic_info.current_position,
            },
            score: xml_analysis.score,
            summary: xml_analysis.summary,
            skills: Skills {
                level: xml_analysis.skills.level,
                details: xml_analysis.skills.details,
            },
            experience: Experience {
                level: xml_analysis.experience.level,
                details: xml_analysis.experience.details,
            },
            strengths: xml_analysis.strengths.items,
            concerns: xml_analysis.concerns.items,
            focus: xml_analysis.focus.items,
        })
    }
}
