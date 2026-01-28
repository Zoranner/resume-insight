use anyhow::{Context, Result};
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub llm: LlmConfig,
    pub server: ServerConfig,
}

#[derive(Debug, Clone)]
pub struct LlmConfig {
    pub base_url: String,
    pub model: String,
    pub api_key: String,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub data_dir: String,
    pub log_dir: String,
    pub base_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            llm: LlmConfig {
                base_url: env::var("LLM_BASE_URL").context("LLM_BASE_URL not set")?,
                model: env::var("LLM_MODEL").context("LLM_MODEL not set")?,
                api_key: env::var("LLM_API_KEY").context("LLM_API_KEY not set")?,
            },
            server: ServerConfig {
                data_dir: env::var("DATA_DIR").unwrap_or_else(|_| "./data/files".to_string()),
                log_dir: env::var("LOG_DIR").unwrap_or_else(|_| "./logs".to_string()),
                base_url: env::var("SERVER_BASE_URL")
                    .context("SERVER_BASE_URL not set (e.g., http://localhost:3000)")?,
            },
        })
    }
}
