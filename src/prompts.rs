use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// 系统提示词（内置）
const SYSTEM_PROMPT: &str = "你是一位专业的HR和招聘专家，擅长分析简历并给出客观、专业的评价。你需要根据岗位要求评估候选人的匹配度。";

/// XML 输出格式规范（固定，不可配置）
const OUTPUT_FORMAT_SPEC: &str = r#"
## 返回格式

严格的 XML 格式，不要有任何额外的文字说明：

```xml
<analysis>
  <basic_info>
    <name>张三</name>
    <gender>男</gender>
    <age>28</age>
    <phone>13812345678</phone>
    <email>zhangsan@example.com</email>
    <location>北京</location>
    <work_years>5年</work_years>
    <degree>本科</degree>
    <major>计算机科学与技术</major>
    <school>北京大学</school>
    <current_company>某科技公司</current_company>
    <current_position>高级后端工程师</current_position>
  </basic_info>
  
  <score>85</score>
  <summary>候选人具有5年软件开发经验，技术栈扎实。在分布式系统和高并发场景有丰富实践，主导过多个核心项目。Rust 技能突出，符合岗位核心要求。有开源贡献，展现良好的技术影响力。整体与岗位匹配度较高，建议进入面试环节。</summary>
  
  <skills>
    <level>优秀</level>
    <details>精通 Rust 语言，熟悉 Tokio 异步编程。对分布式系统、微服务架构有深入理解。数据库和缓存使用经验丰富。完全符合岗位技术栈要求。</details>
  </skills>
  
  <experience>
    <level>良好</level>
    <details>5年后端开发经验，参与过3个大型项目。有从0到1搭建系统的经验，能独立承担核心模块开发。项目复杂度适中,展现出较强的工程能力。</details>
  </experience>
  
  <strengths>
    <item>Rust 技术深度突出，有3年以上实战经验和开源项目贡献</item>
    <item>分布式系统设计能力强，主导过千万级用户量的核心服务</item>
    <item>代码质量意识好，注重测试和文档，有良好的工程素养</item>
    <item>学习能力强，技术栈更新及时，保持技术敏感度</item>
  </strengths>
  
  <concerns>
    <item>团队协作经验描述较少，需面试中重点了解沟通协作能力</item>
    <item>云原生技术栈（K8s）经验不足，需确认学习意愿和能力</item>
    <item>简历中性能优化案例缺少量化数据，需验证实际深度</item>
  </concerns>
  
  <focus>
    <item>深入考察分布式系统设计能力：询问具体架构决策、技术选型依据、遇到的挑战及解决方案</item>
    <item>验证 Rust 实战能力：了解异步编程最佳实践、内存管理经验、性能调优案例</item>
    <item>评估问题解决能力：询问遇到的最大技术难题、分析思路、解决过程和反思总结</item>
    <item>了解团队协作方式：沟通风格、code review 习惯、技术分享经验、冲突处理方式</item>
    <item>确认岗位匹配度：对该岗位的理解、职业规划、技术成长预期、稳定性评估</item>
  </focus>
</analysis>
```

**重要提示**：所有基础信息字段必须从简历中真实提取，如果简历中没有相关信息，必须填写"未知"，不要编造或推测。
"#;

/// 提示词管理器
pub struct PromptManager {
    jobs_cache: HashMap<String, String>,
}

impl PromptManager {
    /// 从文件加载提示词和岗位配置
    pub fn load() -> Result<Self> {
        // 预加载所有岗位文件
        let jobs_cache = Self::load_all_jobs()?;

        Ok(Self { jobs_cache })
    }

    /// 加载所有岗位文件
    fn load_all_jobs() -> Result<HashMap<String, String>> {
        let jobs_dir = Path::new("prompts/jobs");
        let mut jobs = HashMap::new();

        if !jobs_dir.exists() {
            anyhow::bail!("Jobs directory not found at prompts/jobs");
        }

        for entry in fs::read_dir(jobs_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                let job_key = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .context("Invalid job filename")?
                    .to_string();

                let content = fs::read_to_string(&path)
                    .context(format!("Failed to read job file: {:?}", path))?;

                jobs.insert(job_key, content);
            }
        }

        if jobs.is_empty() {
            anyhow::bail!("No job files found in prompts/jobs/ directory");
        }

        Ok(jobs)
    }

    /// 获取系统提示词
    pub fn get_system_prompt(&self) -> &str {
        SYSTEM_PROMPT
    }

    /// 为视觉模型构建分析提示词（不需要传入简历内容，由模型直接从文件提取）
    pub fn build_analysis_prompt_for_vision(&self, job_key: Option<&str>) -> Result<String> {
        // 获取岗位要求
        let job_key = job_key.unwrap_or("default");
        let job_content = self
            .jobs_cache
            .get(job_key)
            .or_else(|| self.jobs_cache.get("default"))
            .context(format!("Job '{}' not found", job_key))?;

        // 格式化岗位要求为 XML
        let job_xml = format!(
            "<job_title>{}</job_title>\n<requirements>\n{}\n</requirements>",
            Self::extract_title(job_content),
            job_content.trim()
        );

        // 构建视觉模型专用提示词
        let vision_instructions = format!(
            r#"请仔细阅读上传的简历文件，并根据以下岗位要求进行分析：

{}

{}

请注意：
1. 仔细提取简历中的所有关键信息，包括基础信息、技能、经验等
2. 如果简历中没有某个基础信息字段，请填写"未知"，不要推测或编造
3. 按照 XML 格式严格输出分析结果"#,
            job_xml, OUTPUT_FORMAT_SPEC
        );

        Ok(vision_instructions)
    }

    /// 从 Markdown 内容提取标题
    fn extract_title(content: &str) -> String {
        content
            .lines()
            .find(|line| line.starts_with("# "))
            .map(|line| line.trim_start_matches("# ").trim().to_string())
            .unwrap_or_else(|| "未知岗位".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_prompts() {
        let manager = PromptManager::load();
        assert!(manager.is_ok());
    }

    #[test]
    fn test_build_prompt_for_vision() {
        let manager = PromptManager::load().unwrap();

        let prompt = manager.build_analysis_prompt_for_vision(Some("rust-backend-engineer"));

        assert!(prompt.is_ok());
        let prompt = prompt.unwrap();
        assert!(prompt.contains("<job_title>"));
        assert!(prompt.contains("<analysis>"));
    }
}
