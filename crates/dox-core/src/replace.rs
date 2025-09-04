//! AI-powered smart text replacement
//!
//! This module provides intelligent text replacement capabilities using AI
//! to understand context and provide more accurate replacements.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::generate::{
    claude::ClaudeProvider, openai::OpenAIProvider, ContentGenerator, ContentType,
    GenerationRequest,
};

/// AI-powered smart replacement engine
pub struct SmartReplacer {
    ai_provider: Box<dyn ContentGenerator>,
    context: Option<String>,
}

impl SmartReplacer {
    /// Create a new smart replacer with the specified AI model
    pub fn new(model: String, api_key: String, context: Option<String>) -> Result<Self> {
        let ai_provider: Box<dyn ContentGenerator> = if model.starts_with("gpt-") {
            Box::new(OpenAIProvider::new(api_key))
        } else if model.starts_with("claude-") {
            Box::new(ClaudeProvider::new(api_key))
        } else {
            return Err(anyhow::anyhow!("지원되지 않는 AI 모델: {}", model));
        };

        Ok(SmartReplacer {
            ai_provider,
            context,
        })
    }

    /// Generate smart replacement suggestions
    pub async fn suggest_replacement(
        &self,
        original_text: &str,
        target_replacement: &str,
        document_context: &str,
    ) -> Result<String> {
        let prompt =
            self.build_replacement_prompt(original_text, target_replacement, document_context);

        let request = GenerationRequest {
            prompt,
            content_type: ContentType::Custom,
            model: "gpt-3.5-turbo".to_string(), // TODO: Make configurable
            max_tokens: 500,
            temperature: 0.3, // Lower temperature for more precise replacements
            language: "ko".to_string(),
            audience: "전문가".to_string(),
            tone: "정확한".to_string(),
            context: self.context.clone(),
            stream: false,
            provider_params: HashMap::new(),
        };

        let response = self.ai_provider.generate(&request).await?;
        Ok(response.content.trim().to_string())
    }

    /// Build a prompt for AI replacement
    fn build_replacement_prompt(
        &self,
        original_text: &str,
        target_replacement: &str,
        document_context: &str,
    ) -> String {
        let context_info = match &self.context {
            Some(ctx) => format!("\n문서 유형: {}", ctx),
            None => String::new(),
        };

        format!(
            "다음 텍스트를 문맥에 맞게 자연스럽게 교체해주세요.\n\n\
            원본 텍스트: \"{}\"\n\
            교체할 텍스트: \"{}\"\n\
            문서 문맥: {}{}\n\n\
            요구사항:\n\
            1. 문맥과 어조를 유지하며 자연스럽게 교체\n\
            2. 문법과 표현을 정확하게 맞춤\n\
            3. 전문적이고 일관성 있는 표현 사용\n\
            4. 교체된 텍스트만 출력 (설명 없이)\n\n\
            교체된 텍스트:",
            original_text, target_replacement, document_context, context_info
        )
    }

    /// Analyze text context to provide better replacements
    pub async fn analyze_context(&self, text_context: &str) -> Result<ContextAnalysis> {
        let prompt = format!(
            "다음 텍스트의 문맥을 분석하여 다음 정보를 JSON 형식으로 제공해주세요:\n\n\
            텍스트: \"{}\"\n\n\
            분석 요소:\n\
            1. tone: 톤 (formal/informal/technical/friendly)\n\
            2. domain: 도메인 (business/technical/academic/personal)\n\
            3. audience: 대상 독자 (executives/developers/general/students)\n\
            4. writing_style: 글쓰기 스타일 (professional/casual/academic)\n\n\
            JSON만 응답해주세요:",
            text_context
        );

        let request = GenerationRequest {
            prompt,
            content_type: ContentType::Custom,
            model: "gpt-3.5-turbo".to_string(),
            max_tokens: 200,
            temperature: 0.1,
            language: "ko".to_string(),
            audience: "전문가".to_string(),
            tone: "분석적".to_string(),
            context: None,
            stream: false,
            provider_params: HashMap::new(),
        };

        let response = self.ai_provider.generate(&request).await?;

        // Try to parse the JSON response
        match serde_json::from_str::<ContextAnalysis>(&response.content) {
            Ok(analysis) => Ok(analysis),
            Err(_) => Ok(ContextAnalysis::default()), // Fallback to default
        }
    }
}

/// Context analysis result
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContextAnalysis {
    pub tone: String,
    pub domain: String,
    pub audience: String,
    pub writing_style: String,
}

impl Default for ContextAnalysis {
    fn default() -> Self {
        ContextAnalysis {
            tone: "formal".to_string(),
            domain: "business".to_string(),
            audience: "general".to_string(),
            writing_style: "professional".to_string(),
        }
    }
}

/// Smart replacement suggestion
#[derive(Debug, Clone)]
pub struct ReplacementSuggestion {
    pub original: String,
    pub suggested: String,
    pub confidence: f32,
    pub reasoning: Option<String>,
}

impl ReplacementSuggestion {
    pub fn new(original: String, suggested: String, confidence: f32) -> Self {
        ReplacementSuggestion {
            original,
            suggested,
            confidence,
            reasoning: None,
        }
    }

    pub fn with_reasoning(mut self, reasoning: String) -> Self {
        self.reasoning = Some(reasoning);
        self
    }
}
