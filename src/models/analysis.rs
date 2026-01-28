use serde::{Deserialize, Serialize};

/// 简历分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Analysis {
    pub basic_info: BasicInfo,
    pub score: u32,
    pub summary: String,
    pub skills: Skills,
    pub experience: Experience,
    pub strengths: Vec<String>,
    pub concerns: Vec<String>,
    pub focus: Vec<String>,
}

/// 候选人基础信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicInfo {
    pub name: String,
    pub gender: String,
    pub age: String,
    pub phone: String,
    pub email: String,
    pub location: String,
    pub work_years: String,
    pub degree: String,
    pub major: String,
    pub school: String,
    pub current_company: String,
    pub current_position: String,
}

/// 技能评估
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skills {
    pub level: String,
    pub details: String,
}

/// 经验评估
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experience {
    pub level: String,
    pub details: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analysis_serialization() {
        let analysis = Analysis {
            basic_info: BasicInfo {
                name: "张三".to_string(),
                gender: "男".to_string(),
                age: "28".to_string(),
                phone: "138****1234".to_string(),
                email: "zhang@example.com".to_string(),
                location: "北京".to_string(),
                work_years: "5年".to_string(),
                degree: "本科".to_string(),
                major: "计算机科学".to_string(),
                school: "北京大学".to_string(),
                current_company: "某公司".to_string(),
                current_position: "高级工程师".to_string(),
            },
            score: 85,
            summary: "优秀候选人".to_string(),
            skills: Skills {
                level: "优秀".to_string(),
                details: "技术栈扎实".to_string(),
            },
            experience: Experience {
                level: "良好".to_string(),
                details: "5年经验".to_string(),
            },
            strengths: vec!["Rust 精通".to_string()],
            concerns: vec!["团队协作待考察".to_string()],
            focus: vec!["架构能力".to_string()],
        };

        let json = serde_json::to_string(&analysis).unwrap();
        assert!(json.contains("\"score\":85"));
        assert!(json.contains("优秀候选人"));
        assert!(json.contains("张三"));
    }

    #[test]
    fn test_analysis_deserialization() {
        let json = r#"{
            "basic_info": {
                "name": "李四",
                "gender": "女",
                "age": "25",
                "phone": "未知",
                "email": "li@example.com",
                "location": "上海",
                "work_years": "3年",
                "degree": "硕士",
                "major": "软件工程",
                "school": "复旦大学",
                "current_company": "某公司",
                "current_position": "工程师"
            },
            "score": 90,
            "summary": "测试",
            "skills": {"level": "优秀", "details": "详情"},
            "experience": {"level": "良好", "details": "详情"},
            "strengths": ["优点1"],
            "concerns": ["关注1"],
            "focus": ["重点1"]
        }"#;

        let analysis: Analysis = serde_json::from_str(json).unwrap();
        assert_eq!(analysis.score, 90);
        assert_eq!(analysis.summary, "测试");
        assert_eq!(analysis.basic_info.name, "李四");
    }
}
