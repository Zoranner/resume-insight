# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- 项目代码结构重构，添加 `models/` 模块
- 添加基础测试框架
- 添加项目文档（CHANGELOG、LICENSE、CONTRIBUTING）

## [0.1.0] - 2024-01-28

### Added
- 初始版本发布
- 支持 PDF/Word/图片等多格式简历解析
- 集成 Textin API 进行文档提取
- 集成 OpenAI 兼容的 LLM API 进行简历分析
- 支持岗位定制化分析
- XML 结构化输出
- 健康检查端点
- 分析接口端点

### Features
- Axum + Tokio 异步架构
- 灵活的提示词管理
- 动态岗位配置

[Unreleased]: https://github.com/yourusername/resume-insight/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/yourusername/resume-insight/releases/tag/v0.1.0
