//! AI content generation core functionality
//!
//! This module provides the core abstractions and traits for AI-powered
//! content generation that will be used by various AI providers.

pub mod claude;
pub mod openai;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Trait for AI content generation providers
#[async_trait::async_trait]
pub trait ContentGenerator: Send + Sync {
    /// Generate content based on the provided request
    async fn generate(&self, request: &GenerationRequest) -> Result<GenerationResponse>;

    /// Get the provider name
    fn provider_name(&self) -> &str;

    /// Get supported models for this provider
    fn supported_models(&self) -> Vec<String>;

    /// Validate if a model is supported
    fn supports_model(&self, model: &str) -> bool {
        self.supported_models().contains(&model.to_string())
    }
}

/// Request for content generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationRequest {
    /// The prompt for content generation
    pub prompt: String,

    /// Type of content to generate
    pub content_type: ContentType,

    /// AI model to use
    pub model: String,

    /// Maximum number of tokens in the response
    pub max_tokens: usize,

    /// Temperature for randomness (0.0-1.0)
    pub temperature: f32,

    /// Language for the generated content
    pub language: String,

    /// Target audience
    pub audience: String,

    /// Tone of voice
    pub tone: String,

    /// Additional context to include
    pub context: Option<String>,

    /// Whether to stream the response
    pub stream: bool,

    /// Additional parameters for the provider
    pub provider_params: HashMap<String, serde_json::Value>,
}

/// Response from content generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationResponse {
    /// The generated content
    pub content: String,

    /// Model used for generation
    pub model: String,

    /// Provider that generated the content
    pub provider: String,

    /// Token usage information
    pub usage: Option<Usage>,

    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

/// Types of content that can be generated
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    Blog,
    Documentation,
    Report,
    Summary,
    Email,
    Proposal,
    Custom,
}

impl ContentType {
    pub fn as_str(&self) -> &str {
        match self {
            ContentType::Blog => "blog",
            ContentType::Documentation => "documentation",
            ContentType::Report => "report",
            ContentType::Summary => "summary",
            ContentType::Email => "email",
            ContentType::Proposal => "proposal",
            ContentType::Custom => "custom",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            ContentType::Blog => "Blog post or article",
            ContentType::Documentation => "Technical documentation",
            ContentType::Report => "Technical report or analysis",
            ContentType::Summary => "Summary of existing content",
            ContentType::Email => "Email or message",
            ContentType::Proposal => "Business proposal",
            ContentType::Custom => "Custom content",
        }
    }

    pub fn default_instructions(&self) -> &str {
        match self {
            ContentType::Blog => {
                "Write an engaging blog post with a compelling title, introduction, \
                well-structured body sections, and conclusion. Use a conversational \
                tone and include relevant examples."
            }
            ContentType::Documentation => {
                "Create comprehensive technical documentation that is clear, accurate, \
                and well-organized. Include proper headings, code examples where \
                applicable, and step-by-step instructions."
            }
            ContentType::Report => {
                "Generate a professional technical report with executive summary, \
                detailed analysis, findings, and recommendations. Include data, \
                evidence-based conclusions, and actionable insights."
            }
            ContentType::Summary => {
                "Create a concise summary that captures the key points, main ideas, \
                and important details while maintaining the essential information."
            }
            ContentType::Email => {
                "Write a professional email with an appropriate subject line, greeting, \
                clear message body, and proper closing."
            }
            ContentType::Proposal => {
                "Create a business proposal with problem statement, proposed solution, \
                benefits, timeline, and next steps."
            }
            ContentType::Custom => {
                "Generate content according to the specific requirements provided."
            }
        }
    }
}

/// Template engine for prompt customization
pub struct TemplateEngine {
    variables: HashMap<String, String>,
}

impl TemplateEngine {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub fn with_variables(variables: HashMap<String, String>) -> Self {
        Self { variables }
    }

    pub fn add_variable(&mut self, key: String, value: String) {
        self.variables.insert(key, value);
    }

    pub fn render(&self, template: &str) -> Result<String> {
        let mut result = template.to_string();

        // Replace variables in {{key}} format
        for (key, value) in &self.variables {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        // Check for unresolved placeholders
        if result.contains("{{") && result.contains("}}") {
            let unresolved: Vec<&str> = result
                .split("{{")
                .skip(1)
                .filter_map(|s| s.split("}}").next())
                .collect();

            if !unresolved.is_empty() {
                return Err(anyhow!(
                    "Template contains unresolved placeholders: {}",
                    unresolved.join(", ")
                ));
            }
        }

        Ok(result)
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Built-in templates for different content types
pub struct BuiltinTemplates;

impl BuiltinTemplates {
    pub fn get_template(content_type: ContentType, language: &str) -> String {
        match (content_type, language) {
            // Korean templates
            (ContentType::Blog, "ko") => {
                "당신은 전문 콘텐츠 작가입니다. 다음 주제에 대해 매력적인 블로그 포스트를 작성하세요: {{prompt}}\n\n\
                대상 독자: {{audience}}\n\
                톤: {{tone}}\n\
                언어: 한국어\n\n\
                구조:\n\
                1. 제목 (흥미로운 제목으로 독자의 관심을 끌어야 함)\n\
                2. 서론 (주제 소개와 독자에게 주는 가치)\n\
                3. 본문 (소제목을 활용한 체계적인 내용 구성)\n\
                4. 결론 (핵심 내용 요약과 행동 유도)\n\n\
                참고사항: 한국어 표현을 자연스럽게 사용하고, 독자와의 소통을 중시하는 친근한 문체를 사용하세요.".to_string()
            }
            (ContentType::Documentation, "ko") => {
                "당신은 기술 문서 작성 전문가입니다. 다음에 대한 명확하고 포괄적인 기술 문서를 작성하세요: {{prompt}}\n\n\
                대상 독자: {{audience}}\n\
                톤: {{tone}}\n\
                언어: 한국어\n\n\
                포함 요소:\n\
                1. 개요 (기능 또는 시스템 소개)\n\
                2. 전제 조건 (필요한 환경, 도구, 지식)\n\
                3. 단계별 설명 (실행 가능한 구체적 방법)\n\
                4. 코드 예제 (필요시 실제 코드 포함)\n\
                5. 문제 해결 (자주 발생하는 오류와 해결책)\n\n\
                참고사항: 기술적 정확성을 유지하면서도 이해하기 쉬운 한국어 표현을 사용하세요.".to_string()
            }
            (ContentType::Report, "ko") => {
                "당신은 전문 분석가입니다. 다음 주제에 대한 포괄적인 기술 보고서를 작성하세요: {{prompt}}\n\n\
                대상 독자: {{audience}}\n\
                톤: {{tone}}\n\
                언어: 한국어\n\n\
                보고서 구성:\n\
                1. 개요 (보고서 목적과 범위)\n\
                2. 분석 (데이터 기반의 체계적 분석)\n\
                3. 결과 (주요 발견사항과 인사이트)\n\
                4. 제안 (실행 가능한 권장사항과 다음 단계)\n\n\
                참고사항: 객관적이고 전문적인 한국어를 사용하여 신뢰성 있는 보고서를 작성하세요.".to_string()
            }
            (ContentType::Email, "ko") => {
                "전문적인 비즈니스 이메일을 작성하세요: {{prompt}}\n\n\
                수신자: {{audience}}\n\
                톤: {{tone}}\n\
                언어: 한국어\n\n\
                이메일 구성:\n\
                1. 제목 (명확하고 구체적인 제목)\n\
                2. 인사말 (적절한 존칭과 인사)\n\
                3. 본문 (명확하고 간결한 메시지)\n\
                4. 다음 단계 (필요한 조치사항)\n\
                5. 마무리 인사 (정중한 마무리와 서명)\n\n\
                참고사항: 한국의 비즈니스 이메일 예의를 지키면서 명확한 의사소통을 하세요.".to_string()
            }
            (ContentType::Proposal, "ko") => {
                "다음에 대한 비즈니스 제안서를 작성하세요: {{prompt}}\n\n\
                대상: {{audience}}\n\
                톤: {{tone}}\n\
                언어: 한국어\n\n\
                제안서 구성:\n\
                1. 요약 (제안의 핵심 내용)\n\
                2. 문제 정의 (현재 상황과 해결해야 할 과제)\n\
                3. 해결책 (구체적인 솔루션 제시)\n\
                4. 기대 효과 (예상되는 이익과 성과)\n\
                5. 실행 계획 (단계별 추진 방안과 일정)\n\
                6. 다음 단계 (승인 후 진행 과정)\n\n\
                참고사항: 설득력 있고 전문적인 한국어로 제안의 가치를 명확히 전달하세요.".to_string()
            }
            (ContentType::Summary, "ko") => {
                "다음 내용의 핵심 요약을 한국어로 작성하세요:\n\n{{prompt}}\n\n\
                요약 시 주의사항:\n\
                - 주요 포인트와 핵심 아이디어를 놓치지 말고 포함\n\
                - 원문의 의도와 맥락을 유지\n\
                - 간결하면서도 완전한 정보 전달\n\
                - 자연스러운 한국어 표현 사용".to_string()
            }
            (ContentType::Custom, "ko") => {
                "{{prompt}}\n\n한국어로 자연스럽고 품질 높은 콘텐츠를 작성해주세요.".to_string()
            }
            // English templates (fallback)
            (ContentType::Blog, _) => {
                "You are a professional content writer. Write an engaging blog post about: {{prompt}}\n\n\
                Target audience: {{audience}}\n\
                Tone: {{tone}}\n\
                Language: {{language}}\n\n\
                Structure:\n\
                1. Title (compelling and attention-grabbing)\n\
                2. Introduction (topic introduction and value proposition)\n\
                3. Main content (well-structured with subheadings)\n\
                4. Conclusion (summary and call-to-action)\n\n\
                Please write in a natural, engaging style that resonates with the target audience.".to_string()
            }
            (ContentType::Documentation, _) => {
                "You are a technical writer creating comprehensive documentation. Write clear technical documentation for: {{prompt}}\n\n\
                Target audience: {{audience}}\n\
                Tone: {{tone}}\n\
                Language: {{language}}\n\n\
                Include:\n\
                1. Overview (feature or system introduction)\n\
                2. Prerequisites (required environment, tools, knowledge)\n\
                3. Step-by-step instructions (actionable detailed methods)\n\
                4. Code examples (actual code when relevant)\n\
                5. Troubleshooting (common errors and solutions)\n\n\
                Maintain technical accuracy while ensuring clarity and accessibility.".to_string()
            }
            (ContentType::Report, _) => {
                "You are a technical analyst creating a comprehensive report. Write a professional technical report about: {{prompt}}\n\n\
                Target audience: {{audience}}\n\
                Tone: {{tone}}\n\
                Language: {{language}}\n\n\
                Report structure:\n\
                1. Executive summary (purpose and scope)\n\
                2. Analysis (systematic data-driven analysis)\n\
                3. Findings (key discoveries and insights)\n\
                4. Recommendations (actionable suggestions and next steps)\n\n\
                Use objective, professional language to ensure credibility and trustworthiness.".to_string()
            }
            (ContentType::Email, _) => {
                "Write a professional email about: {{prompt}}\n\n\
                Target audience: {{audience}}\n\
                Tone: {{tone}}\n\
                Language: {{language}}\n\n\
                Email structure:\n\
                1. Subject line (clear and specific)\n\
                2. Greeting (appropriate salutation)\n\
                3. Body (clear and concise message)\n\
                4. Next steps (required actions)\n\
                5. Professional closing (polite closing and signature)\n\n\
                Follow professional email etiquette while ensuring clear communication.".to_string()
            }
            (ContentType::Proposal, _) => {
                "Write a business proposal for: {{prompt}}\n\n\
                Target audience: {{audience}}\n\
                Tone: {{tone}}\n\
                Language: {{language}}\n\n\
                Proposal structure:\n\
                1. Executive summary (core proposal content)\n\
                2. Problem statement (current situation and challenges)\n\
                3. Solution (specific solution presentation)\n\
                4. Benefits (expected advantages and outcomes)\n\
                5. Implementation plan (phased approach and timeline)\n\
                6. Next steps (post-approval process)\n\n\
                Use persuasive, professional language to clearly convey the proposal's value.".to_string()
            }
            (ContentType::Summary, _) => {
                "Create a concise summary of the following content in {{language}}:\n\n{{prompt}}\n\n\
                Summary guidelines:\n\
                - Include key points and main ideas\n\
                - Maintain original intent and context\n\
                - Provide complete information concisely\n\
                - Use natural language expressions".to_string()
            }
            (ContentType::Custom, _) => {
                "{{prompt}}\n\nPlease create high-quality content in {{language}}.".to_string()
            }
        }
    }
}
