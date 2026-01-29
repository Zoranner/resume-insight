use anyhow::{Context, Result};
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub llm: LlmConfig,
    pub server: ServerConfig,
    pub database: DatabaseConfig,
}

#[derive(Debug, Clone)]
pub struct LlmConfig {
    pub base_url: String,
    pub model: String,
    pub api_key: String,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub files_dir: String,
    pub logs_dir: String,
    pub base_url: String,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
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
                files_dir: env::var("FILES_DIR").unwrap_or_else(|_| "./data/files".to_string()),
                logs_dir: env::var("LOGS_DIR").unwrap_or_else(|_| "./logs".to_string()),
                base_url: env::var("SERVER_BASE_URL")
                    .context("SERVER_BASE_URL not set (e.g., http://localhost:3000)")?,
            },
            database: DatabaseConfig {
                url: env::var("DATABASE_URL")
                    .unwrap_or_else(|_| "sqlite://data/resume.db?mode=rwc".to_string()),
            },
        })
    }
}
