//! AI content generation core functionality
//!
//! This module provides the core abstractions and traits for AI-powered
//! content generation that will be used by various AI providers.

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
        // For now, use simple templates until template files are properly set up
        match (content_type, language) {
            (ContentType::Blog, _) => {
                format!(
                    "You are a professional content writer. Write an engaging blog post about: {{{{prompt}}}}\n\
                    Target audience: {{{{audience}}}}\n\
                    Tone: {{{{tone}}}}\n\
                    Language: {{{{language}}}}\n\n\
                    Structure: Title, Introduction, Main content with subheadings, Conclusion with call-to-action."
                )
            }
            (ContentType::Documentation, _) => {
                format!(
                    "You are a technical writer creating comprehensive documentation. Write clear technical documentation for: {{{{prompt}}}}\n\
                    Target audience: {{{{audience}}}}\n\
                    Tone: {{{{tone}}}}\n\
                    Language: {{{{language}}}}\n\n\
                    Include: Overview, Prerequisites, Step-by-step instructions, Examples, Troubleshooting."
                )
            }
            (ContentType::Report, _) => {
                format!(
                    "You are a technical analyst creating a comprehensive report. Write a professional technical report about: {{{{prompt}}}}\n\
                    Target audience: {{{{audience}}}}\n\
                    Tone: {{{{tone}}}}\n\
                    Language: {{{{language}}}}\n\n\
                    Include: Executive summary, Analysis, Findings, Recommendations."
                )
            }
            (ContentType::Email, _) => {
                format!(
                    "Write a professional email about: {{{{prompt}}}}\n\
                    Target audience: {{{{audience}}}}\n\
                    Tone: {{{{tone}}}}\n\
                    Language: {{{{language}}}}\n\n\
                    Include: Subject line, Greeting, Clear message, Next steps, Professional closing."
                )
            }
            (ContentType::Proposal, _) => {
                format!(
                    "Write a business proposal for: {{{{prompt}}}}\n\
                    Target audience: {{{{audience}}}}\n\
                    Tone: {{{{tone}}}}\n\
                    Language: {{{{language}}}}\n\n\
                    Include: Executive summary, Problem statement, Solution, Benefits, Implementation plan, Next steps."
                )
            }
            (ContentType::Summary, _) => {
                "Create a concise summary of the following content:\n\n{{prompt}}".to_string()
            }
            (ContentType::Custom, _) => "{{prompt}}".to_string(),
        }
    }
}
