# 提示词配置

## 目录结构

```
prompts/
├── resume_analysis.md     # 分析提示词（可随时修改）
└── jobs/                  # 岗位配置
    ├── default.md
    ├── frontend-engineer.md
    ├── fullstack-engineer.md
    ├── devops-engineer.md
    └── rust-backend-engineer.md
```

## 添加新岗位

在 `jobs/` 目录创建 `.md` 文件，文件名即岗位 key（小写+连字符）。

**示例：`senior-backend-engineer.md`**

```markdown
# 岗位名称

## 岗位职责
- 职责 1
- 职责 2

## 任职要求
必须：
- 要求 1
- 要求 2

加分项：
- 加分项 1
- 加分项 2

## 技术栈
- 技术 1
- 技术 2

## 特别关注
- 重点评估项
```

**使用：**

```bash
curl -X POST http://localhost:3000/api/v1/analyze \
  -F "file=@resume.pdf" \
  -F "job=senior-backend-engineer"
```

## 优化提示词

编辑 `resume_analysis.md` 可优化分析指令，无需重启服务。

## 现有岗位

- `default.md` - 默认通用评估
- `frontend-engineer.md` - 前端工程师
- `fullstack-engineer.md` - 全栈工程师
- `devops-engineer.md` - DevOps 工程师
- `rust-backend-engineer.md` - Rust 后端工程师
