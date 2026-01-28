use serde::{Deserialize, Serialize};

/// LLM 聊天请求
#[derive(Debug, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<ThinkingConfig>,
}

/// 深度思考配置
#[derive(Debug, Serialize)]
pub struct ThinkingConfig {
    #[serde(rename = "type")]
    pub thinking_type: String,
}

/// 消息（支持多模态）
#[derive(Debug, Serialize)]
pub struct Message {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<MessageContent>,
}

/// 消息内容（支持文本和多模态）
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    MultiModal(Vec<ContentPart>),
}

/// 内容部分（文本或文件）
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum ContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "file_url")]
    FileUrl { file_url: FileUrl },
}

/// 文件 URL
#[derive(Debug, Serialize)]
pub struct FileUrl {
    pub url: String,
}

/// LLM 聊天响应
#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub choices: Vec<Choice>,
}

/// 选择项
#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: ResponseMessage,
}

/// 响应消息
#[derive(Debug, Deserialize)]
pub struct ResponseMessage {
    pub content: String,
}
