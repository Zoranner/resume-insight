# Resume Insight API

> 基于 Rust + Axum 的智能简历分析 API 服务

[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org/) [![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## ✨ 特性

- 🚀 高性能异步架构（Axum + Tokio）
- 📄 多格式文档解析（PDF、Word、图片等）
- 🤖 OpenAI 兼容接口（支持主流国内大模型）
- 🎯 岗位定制化分析，XML 结构化输出
- 🔧 提示词和岗位配置动态调整

## 📦 快速开始

```bash
# 1. 配置环境变量
cp .env.example .env
# 编辑 .env 填入 API 密钥

# 2. 运行服务
cargo run --release
```

服务启动于 `http://0.0.0.0:3000`

## 🔌 API 使用

### 健康检查

```bash
curl http://localhost:3000/health
```

### 分析简历

```bash
curl -X POST http://localhost:3000/api/v1/analyze \
  -F "file=@resume.pdf" \
  -F "job=your-job-key"
```

> `job` 参数必填，对应 `prompts/jobs/` 目录下的岗位配置文件名（不含 `.md` 后缀）

**响应示例**：

```json
{
  "filename": "resume.pdf",
  "page_count": 2,
  "job_key": "your-job-key",
  "analysis": {
    "score": 85,
    "summary": "候选人具有5年软件开发经验，技术栈扎实。在分布式系统和高并发场景有丰富实践...",
    "skills": {
      "level": "优秀",
      "details": "精通 Rust 语言，熟悉 Tokio 异步编程。对分布式系统、微服务架构有深入理解..."
    },
    "experience": {
      "level": "良好",
      "details": "5年后端开发经验，参与过3个大型项目。有从0到1搭建系统的经验..."
    },
    "strengths": [
      "Rust 技术深度突出，有3年以上实战经验和开源项目贡献",
      "分布式系统设计能力强，主导过千万级用户量的核心服务",
      "代码质量意识好，注重测试和文档"
    ],
    "concerns": [
      "团队协作经验描述较少，需面试中重点了解沟通协作能力",
      "云原生技术栈（K8s）经验不足，需确认学习意愿和能力"
    ],
    "focus": [
      "深入考察分布式系统设计能力：询问具体架构决策、技术选型依据",
      "验证 Rust 实战能力：了解异步编程最佳实践、性能调优案例",
      "评估问题解决能力：询问遇到的最大技术难题及解决方案"
    ]
  }
}
```

## 🛠️ 开发指南

### 项目结构

```
resume-insight/
├── src/
│   ├── main.rs              # 服务入口
│   ├── config.rs            # 配置管理
│   ├── error.rs             # 错误处理
│   ├── handlers.rs          # API 处理器
│   ├── prompts.rs           # 提示词管理
│   ├── models/              # 数据模型
│   │   ├── mod.rs
│   │   ├── analysis.rs      # 分析结果模型
│   │   ├── response.rs      # API 响应模型
│   │   ├── resume.rs        # 简历模型
│   │   ├── llm.rs           # LLM API 模型
│   │   └── textin.rs        # Textin API 模型
│   └── services/
│       ├── mod.rs
│       ├── extractor.rs     # 文档提取服务（Textin）
│       └── analyzer.rs      # LLM 分析服务
├── prompts/
│   ├── README.md            # 提示词使用指南
│   ├── resume_analysis.md   # 分析提示词模板
│   └── jobs/                # 岗位配置目录
│       ├── default.md       # 默认通用评估
│       ├── frontend-engineer.md
│       ├── fullstack-engineer.md
│       ├── devops-engineer.md
│       └── rust-backend-engineer.md
├── tests/
│   ├── README.md            # 测试指南
│   └── integration_test.rs  # 集成测试
├── Cargo.toml
├── .env.example
├── .gitignore
├── Dockerfile
├── README.md
├── LICENSE
├── CHANGELOG.md
└── CONTRIBUTING.md
```

### 添加新岗位

在 `prompts/jobs/` 目录下创建新的 `.md` 文件：

```markdown
# 岗位名称

## 岗位职责

- 职责1
- 职责2

## 任职要求

- 要求1
- 要求2

## 加分项

- 加分项1
- 加分项2
```

文件名即为岗位 key，如 `backend-engineer.md` 对应 `job=backend-engineer`。

### 调整提示词

编辑 `prompts/resume_analysis.md`，支持变量：`{{job_requirements}}`、`{{candidate_resume}}`

## 📝 配置说明

### 环境变量

| 变量名 | 必填 | 说明 |
|--------|------|------|
| `LLM_BASE_URL` | ✅ | LLM API 地址（OpenAI 兼容格式） |
| `LLM_MODEL` | ✅ | 模型名称 |
| `LLM_API_KEY` | ✅ | LLM API 密钥 |
| `TEXTIN_APP_ID` | ✅ | Textin API 应用 ID |
| `TEXTIN_SECRET_CODE` | ✅ | Textin API 密钥 |
| `RUST_LOG` | ❌ | 日志级别（默认：info） |

## 🚀 部署

```bash
# 生产编译
cargo build --release

# 运行（确保 prompts/ 目录和 .env 在同一目录）
./target/release/resume-insight
```

## 📄 许可证

MIT License
