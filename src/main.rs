mod config;
mod error;
mod handlers;
mod logger;
mod models;
mod prompts;
mod services;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::{
    cors::CorsLayer, limit::RequestBodyLimitLayer, services::ServeDir, trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 加载环境变量
    dotenvy::dotenv().ok();

    // 初始化日志
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,resume_insight=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 加载配置
    let config = config::Config::from_env()?;
    tracing::info!("Configuration loaded successfully");

    // 创建数据文件目录和日志目录
    tokio::fs::create_dir_all(&config.server.data_dir).await?;
    tokio::fs::create_dir_all(&config.server.log_dir).await?;
    tracing::info!("Data directory: {}", config.server.data_dir);
    tracing::info!("Log directory: {}", config.server.log_dir);

    let data_dir = config.server.data_dir.clone();

    // 创建应用状态
    let state = handlers::AppState::new(config)?;

    // 构建路由
    let app = Router::new()
        .route("/health", get(handlers::health_check))
        .route("/api/v1/analyze", post(handlers::analyze_resume))
        .nest_service("/files", ServeDir::new(&data_dir)) // 静态文件服务
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .layer(RequestBodyLimitLayer::new(500 * 1024 * 1024)) // 500MB
        .with_state(state);

    // 启动服务器
    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::info!("🚀 Resume Insight API running on http://{}", addr);
    tracing::info!("📝 API endpoint: POST http://{}/api/v1/analyze", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
